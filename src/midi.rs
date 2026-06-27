use crate::chord_finder::{ChordFinder, ChordResult};
use anyhow::{Context, Result};
use midir::{MidiInput, MidiInputConnection};
use std::collections::BTreeSet;
use std::pin::Pin;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::{sleep, Instant, Sleep};
use tracing::{debug, error, info, warn};

const DEBOUNCE_DELAY: Duration = Duration::from_millis(30);

#[derive(Debug, Clone, PartialEq)]
pub enum MidiEvent {
    NoteOn(u8),
    NoteOff(u8),
    ControlChange(u8, u8),
}

#[derive(Debug, Clone)]
pub enum MidiCommand {
    Reconnect(String),
    SetChannel(u8),
    ListPorts,
    Close,
}

#[derive(Debug, Clone)]
pub enum MidiOutput {
    Chord(ChordResult),
    ControlChange(u8, u8),
    Ports(Vec<String>),
    Status { connected: bool, port: String },
    ChannelChanged(u8),
}

#[derive(Debug, Clone)]
pub struct MidiBridgeHandle {
    command_tx: mpsc::Sender<MidiCommand>,
}

impl MidiBridgeHandle {
    pub fn new(command_tx: mpsc::Sender<MidiCommand>) -> Self {
        Self { command_tx }
    }

    pub async fn reconnect(&self, port_name: String) -> Result<()> {
        self.command_tx
            .send(MidiCommand::Reconnect(port_name))
            .await
            .context("MIDI bridge task has stopped")
    }

    pub async fn list_ports(&self) -> Result<()> {
        self.command_tx
            .send(MidiCommand::ListPorts)
            .await
            .context("MIDI bridge task has stopped")
    }

    pub async fn close(self) -> Result<()> {
        self.command_tx
            .send(MidiCommand::Close)
            .await
            .context("MIDI bridge task has stopped")
    }

    pub fn reconnect_blocking(&self, port_name: String) -> Result<()> {
        self.command_tx
            .blocking_send(MidiCommand::Reconnect(port_name))
            .context("MIDI bridge task has stopped")
    }

    pub fn list_ports_blocking(&self) -> Result<()> {
        self.command_tx
            .blocking_send(MidiCommand::ListPorts)
            .context("MIDI bridge task has stopped")
    }

    pub fn set_channel_blocking(&self, channel: u8) -> Result<()> {
        self.command_tx
            .blocking_send(MidiCommand::SetChannel(channel))
            .context("MIDI bridge task has stopped")
    }
}

pub fn run_midi_bridge(
    port_name: String,
    channel: u8,
    output_tx: mpsc::Sender<MidiOutput>,
    command_rx: mpsc::Receiver<MidiCommand>,
) -> Result<()> {
    std::thread::Builder::new()
        .name("crowdchoir-midi".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .expect("Failed to create tokio runtime for MIDI thread");
            rt.block_on(async move {
                if let Err(e) = run_midi_bridge_inner(port_name, channel, output_tx, command_rx).await {
                    error!("MIDI bridge error: {}", e);
                }
            });
        })
        .context("Failed to spawn MIDI bridge thread")?;
    Ok(())
}

async fn run_midi_bridge_inner(
    port_name: String,
    channel: u8,
    output_tx: mpsc::Sender<MidiOutput>,
    mut command_rx: mpsc::Receiver<MidiCommand>,
) -> Result<()> {
    let (midi_event_tx, mut midi_event_rx) = mpsc::channel::<MidiEvent>(64);
    let mut bridge = MidiBridgeState {
        port_name,
        channel,
        output_tx,
        midi_event_tx,
        connection: None,
        chord_finder: ChordFinder::new(),
        played_notes: BTreeSet::new(),
        debounce: None,
        last_cc: None,
    };

    // Initial connection attempt
    bridge.try_connect().await;

    loop {
        tokio::select! {
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    MidiCommand::Reconnect(new_port) => {
                        bridge.port_name = new_port;
                        bridge.try_connect().await;
                    }
                    MidiCommand::SetChannel(ch) => {
                        bridge.channel = ch;
                        let _ = bridge.output_tx.send(MidiOutput::ChannelChanged(ch)).await;
                        bridge.try_connect().await;
                    }
                    MidiCommand::ListPorts => {
                        let ports = list_ports_internal();
                        let _ = bridge.output_tx.send(MidiOutput::Ports(ports)).await;
                    }
                    MidiCommand::Close => {
                        info!("MIDI bridge closing");
                        bridge.connection = None;
                        break;
                    }
                }
            }
            Some(event) = midi_event_rx.recv() => {
                bridge.handle_event(event).await;
            }
            _ = async { bridge.debounce.as_mut().unwrap().as_mut().await }, if bridge.debounce.is_some() => {
                bridge.detect_chord().await;
            }
        }
    }

    Ok(())
}

struct MidiBridgeState {
    port_name: String,
    channel: u8,
    output_tx: mpsc::Sender<MidiOutput>,
    midi_event_tx: mpsc::Sender<MidiEvent>,
    connection: Option<MidiInputConnection<()>>,
    chord_finder: ChordFinder,
    played_notes: BTreeSet<u8>,
    debounce: Option<Pin<Box<Sleep>>>,
    last_cc: Option<(u8, u8, Instant)>,
}

impl MidiBridgeState {
    async fn try_connect(&mut self) {
        self.connection = None;

        let midi_in = match MidiInput::new("crowdchoir midi input") {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to create MIDI input: {}", e);
                self.send_status(false).await;
                return;
            }
        };

        let ports = midi_in.ports();
        let available_names: Vec<String> = ports
            .iter()
            .filter_map(|p| midi_in.port_name(p).ok())
            .collect();

        if available_names.is_empty() {
            warn!(
                "No MIDI input ports available; cannot connect to '{}'",
                self.port_name
            );
            self.send_status(false).await;
            return;
        }

        debug!("Available MIDI input ports: {:?}", available_names);

        let matched_port = ports.iter().find(|p| {
            midi_in
                .port_name(p)
                .map(|name| name == self.port_name || name.contains(&self.port_name))
                .unwrap_or(false)
        });

        let port = match matched_port {
            Some(p) => p,
            None => {
                warn!(
                    "MIDI port '{}' not found; available: {:?}",
                    self.port_name, available_names
                );
                self.send_status(false).await;
                return;
            }
        };

        let port_display_name = midi_in.port_name(port).unwrap_or_else(|_| self.port_name.clone());
        let event_tx = self.midi_event_tx.clone();
        let channel = self.channel;

        match midi_in.connect(
            port,
            "crowdchoir",
            move |_timestamp, message, _data| {
                if let Some(event) = parse_midi(message, channel) {
                    if let Err(e) = event_tx.blocking_send(event) {
                        warn!("MIDI event channel closed: {}", e);
                    }
                }
            },
            (),
        ) {
            Ok(conn) => {
                info!("Connected to MIDI input port: {}", port_display_name);
                self.connection = Some(conn);
                self.send_status(true).await;
            }
            Err(e) => {
                error!("Failed to connect to MIDI port '{}': {}", port_display_name, e);
                self.send_status(false).await;
            }
        }
    }

    async fn send_status(&self, connected: bool) {
        let _ = self
            .output_tx
            .send(MidiOutput::Status {
                connected,
                port: self.port_name.clone(),
            })
            .await;
    }

    async fn handle_event(&mut self, event: MidiEvent) {
        match event {
            MidiEvent::NoteOn(note) => {
                self.played_notes.insert(note);
                debug!("MIDI note on {} (active: {:?})", note, self.played_notes);
                self.debounce = Some(Box::pin(sleep(DEBOUNCE_DELAY)));
            }
            MidiEvent::NoteOff(note) => {
                self.played_notes.remove(&note);
                debug!("MIDI note off {} (active: {:?})", note, self.played_notes);
                self.debounce = Some(Box::pin(sleep(DEBOUNCE_DELAY)));
            }
            MidiEvent::ControlChange(control, value) => {
                let now = Instant::now();
                if let Some((last_control, last_value, last_time)) = self.last_cc {
                    if control == last_control
                        && value == last_value
                        && now.duration_since(last_time) < Duration::from_millis(100)
                    {
                        debug!("Ignoring duplicate MIDI CC {} {}", control, value);
                        return;
                    }
                }
                self.last_cc = Some((control, value, now));
                debug!("MIDI control change {} {}", control, value);
                let _ = self
                    .output_tx
                    .send(MidiOutput::ControlChange(control, value))
                    .await;
            }
        }
    }

    async fn detect_chord(&mut self) {
        if !self.played_notes.is_empty() {
            let notes: Vec<u8> = self.played_notes.iter().copied().collect();
            let result = self.chord_finder.identify_chord(&notes);
            info!("Detected chord: root={} type={}", result.root, result.chord_type);
            let _ = self.output_tx.send(MidiOutput::Chord(result)).await;
        }
        self.played_notes.clear();
        self.debounce = None;
    }
}

fn parse_midi(message: &[u8], channel: u8) -> Option<MidiEvent> {
    if message.is_empty() {
        return None;
    }
    let status = message[0];
    let msg_channel = status & 0x0F;
    if msg_channel != channel {
        return None;
    }

    match status & 0xF0 {
        0x90 if message.len() >= 3 => {
            if message[2] == 0 {
                Some(MidiEvent::NoteOff(message[1]))
            } else {
                Some(MidiEvent::NoteOn(message[1]))
            }
        }
        0x80 if message.len() >= 3 => Some(MidiEvent::NoteOff(message[1])),
        0xB0 if message.len() >= 3 => Some(MidiEvent::ControlChange(message[1], message[2])),
        _ => None,
    }
}

fn list_ports_internal() -> Vec<String> {
    let midi_in = match MidiInput::new("crowdchoir midi list") {
        Ok(m) => m,
        Err(e) => {
            warn!("Failed to create MIDI input for listing ports: {}", e);
            return Vec::new();
        }
    };

    midi_in
        .ports()
        .into_iter()
        .filter_map(|p| midi_in.port_name(&p).ok())
        .collect()
}

pub fn list_midi_ports() -> Vec<String> {
    list_ports_internal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_midi_messages() {
        assert!(matches!(
            parse_midi(&[0x90, 60, 100], 0),
            Some(MidiEvent::NoteOn(60))
        ));
        assert!(matches!(
            parse_midi(&[0x80, 60, 0], 0),
            Some(MidiEvent::NoteOff(60))
        ));
        assert!(matches!(
            parse_midi(&[0x90, 60, 0], 0),
            Some(MidiEvent::NoteOff(60))
        ));
        assert!(matches!(
            parse_midi(&[0xB0, 7, 64], 0),
            Some(MidiEvent::ControlChange(7, 64))
        ));
        assert_eq!(parse_midi(&[0x91, 60, 100], 0), None);
        assert_eq!(parse_midi(&[0x90, 60, 100], 1), None);
    }
}
