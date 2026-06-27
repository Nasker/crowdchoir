use crate::events::{handle_midi_output, on_connect, send_initial_gui_snapshot};
use crate::midi::{list_midi_ports, run_midi_bridge, MidiBridgeHandle, MidiCommand, MidiOutput};
use crate::state::{AppState, GuiStateSnapshot};
use anyhow::{Context, Result};
use axum::body::Body;
use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as HyperBuilder;
use rust_embed::RustEmbed;
use serde_json::json;
use socketioxide::SocketIo;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::RwLock;
use tower::Service;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateAssets;

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub midi_port: String,
    pub midi_channel: u8,
    pub tls: bool,
    pub gui_tx: Option<std::sync::mpsc::Sender<GuiStateSnapshot>>,
    pub midi_cmd_channel: Option<(mpsc::Sender<MidiCommand>, mpsc::Receiver<MidiCommand>)>,
    pub shutdown_rx: Option<oneshot::Receiver<()>>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 5000,
            midi_port: "Driver IAC Bus 1".to_string(),
            midi_channel: 0,
            tls: false,
            gui_tx: None,
            midi_cmd_channel: None,
            shutdown_rx: None,
        }
    }
}

fn detect_local_ip() -> Option<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip())
}

fn make_tls_config() -> Result<rustls::ServerConfig> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut params = rcgen::CertificateParams::default();
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName("localhost".try_into()?),
        rcgen::SanType::IpAddress("127.0.0.1".parse().unwrap()),
    ];
    if let Some(ip) = detect_local_ip() {
        info!("TLS cert: adding LAN IP {} as SAN", ip);
        params.subject_alt_names.push(rcgen::SanType::IpAddress(ip));
    }
    let key_pair = rcgen::KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;
    let cert_der = rustls::pki_types::CertificateDer::from(cert.der().to_vec());
    let key_der = rustls::pki_types::PrivatePkcs8KeyDer::from(key_pair.serialize_der());
    Ok(rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der.into())?)
}

async fn serve_tls(
    listener: TcpListener,
    app: axum::Router,
    tls_cfg: rustls::ServerConfig,
    shutdown_rx: Option<oneshot::Receiver<()>>,
) -> Result<()> {
    use std::pin::pin;
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_cfg));
    let shutdown = pin!(async move {
        if let Some(rx) = shutdown_rx { let _ = rx.await; }
        else { std::future::pending::<()>().await; }
    });
    let mut shutdown = shutdown;
    loop {
        tokio::select! {
            result = listener.accept() => {
                let (tcp, _) = result?;
                let acc = acceptor.clone();
                let svc_app = app.clone();
                tokio::spawn(async move {
                    let tls = match acc.accept(tcp).await {
                        Ok(s) => s,
                        Err(e) => { tracing::warn!("TLS handshake: {e}"); return; }
                    };
                    let io = TokioIo::new(tls);
                    let svc = hyper::service::service_fn(move |req| {
                        let mut app = svc_app.clone();
                        async move { app.call(req.map(axum::body::Body::new)).await }
                    });
                    if let Err(e) = HyperBuilder::new(TokioExecutor::new())
                        .serve_connection_with_upgrades(io, svc)
                        .await
                    {
                        tracing::debug!("Connection closed: {e}");
                    }
                });
            }
            _ = &mut shutdown => {
                info!("Server shutting down gracefully");
                break;
            }
        }
    }
    Ok(())
}

pub async fn run_server(config: ServerConfig) -> Result<()> {
    let address: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .context("Invalid server address")?;

    let mut app_state = AppState::new(config.midi_port.clone(), config.midi_channel);
    app_state.gui_tx = config.gui_tx.clone();
    app_state.midi_ports = tokio::task::spawn_blocking(list_midi_ports)
        .await
        .unwrap_or_default();
    let state = Arc::new(RwLock::new(app_state));

    let (midi_output_tx, mut midi_output_rx) = mpsc::channel::<MidiOutput>(64);
    let (midi_cmd_tx, midi_cmd_rx) = config
        .midi_cmd_channel
        .unwrap_or_else(|| mpsc::channel(16));
    if let Err(e) = run_midi_bridge(
        config.midi_port.clone(),
        config.midi_channel,
        midi_output_tx,
        midi_cmd_rx,
    ) {
        error!("MIDI bridge error: {}", e);
    }
    let midi_handle = MidiBridgeHandle::new(midi_cmd_tx);

    {
        let mut guard = state.write().await;
        guard.midi_handle = Some(midi_handle);
        drop(guard);
    }

    if let Some(gui_tx) = config.gui_tx {
        send_initial_gui_snapshot(&state, &gui_tx);
    }

    let (layer, io) = SocketIo::builder()
        .ping_interval(Duration::from_secs(5))
        .ping_timeout(Duration::from_secs(10))
        .with_state(state.clone())
        .build_layer();

    io.ns("/", on_connect);

    let io_for_midi = io.clone();
    let state_for_midi = state.clone();
    tokio::spawn(async move {
        while let Some(output) = midi_output_rx.recv().await {
            handle_midi_output(output, &io_for_midi, &state_for_midi).await;
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/conductor", get(conductor_handler))
        .route("/static/*path", get(static_handler))
        .layer(layer)
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind(address).await?;
    let shutdown_rx = config.shutdown_rx;

    if config.tls {
        let tls_cfg = make_tls_config()?;
        info!("Server listening on https://{}", address);
        info!("⚠ Self-signed cert — accept the browser security warning on first connect");
        serve_tls(listener, app, tls_cfg, shutdown_rx).await?;
    } else {
        info!("Server listening on http://{}", address);
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                if let Some(rx) = shutdown_rx {
                    let _ = rx.await;
                    info!("Server shutting down gracefully");
                } else {
                    std::future::pending::<()>().await;
                }
            })
            .await?;
    }

    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    match TemplateAssets::get("index.html") {
        Some(file) => {
            let template = String::from_utf8_lossy(file.data.as_ref()).into_owned();
            let html = render_index_template(&template);
            Html(html).into_response()
        }
        None => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

async fn conductor_handler() -> impl IntoResponse {
    match TemplateAssets::get("conductor.html") {
        Some(file) => {
            let html = String::from_utf8_lossy(file.data.as_ref()).into_owned();
            Html(html).into_response()
        }
        None => (StatusCode::NOT_FOUND, "conductor.html not found").into_response(),
    }
}

async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    match StaticAssets::get(&path) {
        Some(file) => {
            let content_type = guess_mime_type(&path);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .body(Body::from(file.data.into_owned()))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap(),
    }
}

fn render_index_template(template: &str) -> String {
    let music_data = json!({
        "chords": serde_json::from_str::<serde_json::Value>(include_str!("../static/data/chords.json")).unwrap(),
        "scales": serde_json::from_str::<serde_json::Value>(include_str!("../static/data/scales.json")).unwrap(),
    });
    let music_data_json = serde_json::to_string(&music_data).unwrap();

    let mut html = template.to_string();
    html = html.replace("{{ music_data | tojson }}", &music_data_json);
    html = html.replace("{{ url_for('static', filename='", "/static/");
    html = html.replace("') }}", "");

    html
}

fn guess_mime_type(path: &str) -> &'static str {
    let path = path.to_lowercase();
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") || path.ends_with(".mjs") {
        "application/javascript"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".mp3") {
        "audio/mpeg"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else if path.ends_with(".otf") {
        "font/otf"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else {
        "application/octet-stream"
    }
}
