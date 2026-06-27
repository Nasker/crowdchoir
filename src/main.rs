use anyhow::Result;
use crowdchoir::midi::{MidiBridgeHandle, MidiCommand};
use crowdchoir::music_controller::{CHORDS_DATA, SCALES_DATA};
use crowdchoir::server::{run_server, ServerConfig};
use crowdchoir::state::GuiStateSnapshot;
use eframe::egui;
use std::collections::VecDeque;
use std::env;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc as async_mpsc;
use tokio::sync::oneshot;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const MAX_LOG_LINES: usize = 20;

// ── Log-capture tracing layer ─────────────────────────────────────────────────

struct GuiLogLayer {
    lines: Arc<Mutex<VecDeque<String>>>,
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for GuiLogLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        struct V(String);
        impl tracing::field::Visit for V {
            fn record_str(&mut self, f: &tracing::field::Field, v: &str) {
                if f.name() == "message" {
                    self.0 = v.to_owned();
                }
            }
            fn record_debug(&mut self, f: &tracing::field::Field, v: &dyn std::fmt::Debug) {
                if f.name() == "message" {
                    self.0 = format!("{v:?}");
                }
            }
        }
        let mut v = V(String::new());
        event.record(&mut v);
        if v.0.is_empty() {
            return;
        }
        let level = *event.metadata().level();
        let entry = format!("[{level}] {}", v.0);
        if let Ok(mut lines) = self.lines.lock() {
            if lines.len() >= MAX_LOG_LINES {
                lines.pop_front();
            }
            lines.push_back(entry);
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

struct CrowdChoirApp {
    state: GuiStateSnapshot,
    rx: Receiver<GuiStateSnapshot>,
    log_lines: Arc<Mutex<VecDeque<String>>>,
    midi_handle: MidiBridgeHandle,
    selected_port: String,
    selected_channel: u8,
    port: u16,
    tls: bool,
    server_running: bool,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl CrowdChoirApp {
    fn new(
        rx: Receiver<GuiStateSnapshot>,
        log_lines: Arc<Mutex<VecDeque<String>>>,
        midi_handle: MidiBridgeHandle,
        port: u16,
        tls: bool,
        shutdown_tx: oneshot::Sender<()>,
    ) -> Self {
        Self {
            state: GuiStateSnapshot::default(),
            rx,
            log_lines,
            midi_handle,
            selected_port: String::new(),
            selected_channel: 0,
            port,
            tls,
            server_running: true,
            shutdown_tx: Some(shutdown_tx),
        }
    }
}

impl eframe::App for CrowdChoirApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain server state updates
        while let Ok(s) = self.rx.try_recv() {
            if self.selected_port.is_empty() {
                self.selected_port = s.midi_port.clone();
            }
            self.selected_channel = s.midi_channel;
            self.state = s;
        }
        if self.selected_port.is_empty() {
            if let Some(p) = self.state.midi_ports.first() {
                self.selected_port = p.clone();
            }
        }

        let chord_label = {
            let root = self.state.current_chord.root as usize;
            let ct = self.state.current_chord.chord_type as usize;
            let rn = SCALES_DATA.root_names.get(root % 12).map_or("?", |s| s.as_str());
            let cn = CHORDS_DATA.names.get(ct).map_or("?", |s| s.as_str());
            format!("{rn} {cn}")
        };

        // ── Top status bar ────────────────────────────────────────────────────
        egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (dot, label) = if self.server_running {
                    (egui::Color32::GREEN, "● Running")
                } else {
                    (egui::Color32::from_rgb(180, 180, 180), "● Stopped")
                };
                ui.colored_label(dot, label);
                ui.weak(format!("  http://localhost:{}", self.port));
                ui.separator();
                ui.label(format!("👥 {}", self.state.connected_clients));
                ui.separator();
                ui.label(format!("🎵 {chord_label}"));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.server_running {
                        if ui.button("⏹  Stop").clicked() {
                            if let Some(tx) = self.shutdown_tx.take() {
                                let _ = tx.send(());
                                self.server_running = false;
                            }
                        }
                    } else {
                        ui.weak("Restart the app to run again");
                    }
                });
            });
        });

        // ── Bottom log tail ───────────────────────────────────────────────────
        egui::TopBottomPanel::bottom("log_panel")
            .min_height(130.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(2.0);
                ui.label(egui::RichText::new("Log").strong().small());
                ui.separator();
                let lines = self.log_lines.lock().unwrap().clone();
                egui::ScrollArea::vertical()
                    .id_source("log_scroll")
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &lines {
                            let color = if line.contains("[ERROR]") {
                                egui::Color32::from_rgb(255, 100, 100)
                            } else if line.contains("[WARN]") {
                                egui::Color32::from_rgb(255, 200, 80)
                            } else {
                                egui::Color32::from_rgb(160, 160, 160)
                            };
                            ui.label(egui::RichText::new(line).monospace().small().color(color));
                        }
                    });
            });

        // ── Central panel: MIDI controls ──────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MIDI Input");
            ui.separator();

            ui.horizontal(|ui| {
                let (dot_color, status) = if self.state.midi_connected {
                    (egui::Color32::GREEN, "Connected")
                } else {
                    (egui::Color32::from_rgb(220, 60, 60), "Disconnected")
                };
                ui.colored_label(dot_color, "●");
                ui.label(status);
                ui.weak(format!(" — {}", self.state.midi_port));
            });

            ui.add_space(10.0);
            ui.label("Port:");
            ui.add_space(2.0);

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("midi_port_combo")
                    .selected_text(if self.selected_port.is_empty() {
                        "(no ports detected)".to_owned()
                    } else {
                        self.selected_port.clone()
                    })
                    .width(ui.available_width() - 130.0)
                    .show_ui(ui, |ui| {
                        for port in &self.state.midi_ports {
                            ui.selectable_value(&mut self.selected_port, port.clone(), port);
                        }
                    });

                if ui.button("Connect").clicked() && !self.selected_port.is_empty() {
                    if let Err(e) = self.midi_handle.reconnect_blocking(self.selected_port.clone()) {
                        tracing::error!("MIDI connect failed: {e}");
                    }
                }

                if ui.button("Refresh").clicked() {
                    if let Err(e) = self.midi_handle.list_ports_blocking() {
                        tracing::error!("MIDI refresh failed: {e}");
                    }
                }
            });

            ui.add_space(6.0);
            ui.label("Channel:");
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                let prev = self.selected_channel;
                egui::ComboBox::from_id_source("midi_channel_combo")
                    .selected_text(format!("Ch {}", self.selected_channel + 1))
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        for ch in 0u8..16 {
                            ui.selectable_value(
                                &mut self.selected_channel,
                                ch,
                                format!("Ch {}", ch + 1),
                            );
                        }
                    });
                if self.selected_channel != prev {
                    if let Err(e) = self.midi_handle.set_channel_blocking(self.selected_channel) {
                        tracing::error!("MIDI set channel failed: {e}");
                    }
                }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(6.0);
            let scheme = if self.tls { "https" } else { "http" };
            ui.weak(format!(
                "Open {}://localhost:{}/conductor in a browser to control the performance.",
                scheme, self.port
            ));
            if self.tls {
                ui.weak("⚠ Self-signed cert — accept the browser warning on first connect");
            }
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let log_lines: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "warn,crowdchoir=info".parse().unwrap());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(GuiLogLayer { lines: log_lines.clone() })
        .init();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5000);
    let midi_port = env::var("MIDI_PORT").unwrap_or_else(|_| "Driver IAC Bus 1".to_string());
    let midi_channel: u8 = env::var("MIDI_CHANNEL")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    let tls = env::var("CROWDCHOIR_TLS").map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false);

    let (gui_tx, gui_rx): (Sender<GuiStateSnapshot>, Receiver<GuiStateSnapshot>) = channel();
    let (midi_cmd_tx, midi_cmd_rx) = async_mpsc::channel::<MidiCommand>(16);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let midi_handle = MidiBridgeHandle::new(midi_cmd_tx.clone());

    let server_config = ServerConfig {
        host,
        port,
        midi_port,
        midi_channel,
        tls,
        gui_tx: Some(gui_tx),
        midi_cmd_channel: Some((midi_cmd_tx, midi_cmd_rx)),
        shutdown_rx: Some(shutdown_rx),
    };

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        if let Err(e) = rt.block_on(run_server(server_config)) {
            tracing::error!("Server exited: {e}");
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 420.0])
            .with_title("CrowdChoir Server"),
        ..Default::default()
    };

    eframe::run_native(
        "CrowdChoir Server",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(CrowdChoirApp::new(
                gui_rx,
                log_lines,
                midi_handle,
                port,
                tls,
                shutdown_tx,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e:?}"))
}
