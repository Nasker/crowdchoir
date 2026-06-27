use anyhow::Result;
use crowdchoir::server::{run_captive_portal, run_server, ServerConfig};
use std::env;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "warn,crowdchoir=info".parse().unwrap());
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5000);
    let midi_port = env::var("MIDI_PORT").unwrap_or_else(|_| "Driver IAC Bus 1".to_string());
    let midi_channel = env::var("MIDI_CHANNEL")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);

    let tls = env::var("CROWDCHOIR_TLS")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    let captive = env::var("CROWDCHOIR_CAPTIVE")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);
    let scheme = if tls { "https" } else { "http" };
    let public_url = env::var("CROWDCHOIR_PUBLIC_URL")
        .unwrap_or_else(|_| format!("{scheme}://{host}:{port}"));

    let config = ServerConfig {
        host,
        port,
        midi_port,
        midi_channel,
        tls,
        gui_tx: None,
        ..Default::default()
    };

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        if captive {
            let captive_addr = ([0, 0, 0, 0], 80).into();
            tokio::spawn(async move {
                if let Err(e) = run_captive_portal(captive_addr, public_url).await {
                    tracing::error!("Captive portal exited: {e}");
                }
            });
        }
        run_server(config).await
    })
}
