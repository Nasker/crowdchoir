use crate::midi::MidiOutput;
use crate::music_controller::{CHORDS_DATA, SCALES_DATA};
use crate::state::{AppState, CurrentChord, GuiStateSnapshot};
use serde::{Deserialize, Serialize};
use socketioxide::extract::{Data, SocketRef, State};
use socketioxide::SocketIo;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

#[derive(Debug, Serialize, Clone)]
pub struct ControlChangePayload {
    pub control: u8,
    pub value: u8,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChordChangedPayload {
    pub root: u8,
    pub chord_type: u8,
}

#[derive(Debug, Serialize, Clone)]
pub struct ClientCountPayload {
    pub count: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct MidiStatusPayload {
    pub connected: bool,
    pub port: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MidiPortsPayload {
    pub ports: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ServerStatePayload {
    pub client_count: usize,
    pub midi_port: String,
    pub midi_connected: bool,
    pub midi_ports: Vec<String>,
    pub current_chord: CurrentChord,
    pub chord_names: Vec<String>,
    pub root_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetChordData {
    pub root: u8,
    pub chord_type: u8,
}

#[derive(Debug, Deserialize)]
pub struct SetMidiPortData {
    pub port: String,
}

#[derive(Debug, Deserialize)]
pub struct ControlChangeData {
    pub control: u8,
    pub value: u8,
}

pub fn on_connect(
    socket: SocketRef,
    State(state): State<Arc<RwLock<AppState>>>,
    io: SocketIo,
) {
    info!("Client connected: {}", socket.id);

    socket.on_disconnect(on_disconnect);
    socket.on("join_conductor", on_join_conductor);
    socket.on("request_midi_ports", on_request_midi_ports);
    socket.on("set_midi_port", on_set_midi_port);
    socket.on("set_chord", on_set_chord);
    socket.on("control_change", on_control_change);

    tokio::spawn(async move {
        let mut guard = state.write().await;
        guard.connected_clients += 1;
        let count = guard.connected_clients;
        guard.notify_gui(true);
        drop(guard);

        let payload = ClientCountPayload { count };
        if let Err(e) = io.to("conductor").emit("client_count", &payload) {
            error!("Failed to emit client_count: {}", e);
        }
    });
}

pub fn on_disconnect(
    socket: SocketRef,
    State(state): State<Arc<RwLock<AppState>>>,
    io: SocketIo,
) {
    info!("Client disconnected: {}", socket.id);
    tokio::spawn(async move {
        let mut guard = state.write().await;
        guard.connected_clients = guard.connected_clients.saturating_sub(1);
        let count = guard.connected_clients;
        guard.notify_gui(true);
        drop(guard);

        let payload = ClientCountPayload { count };
        if let Err(e) = io.to("conductor").emit("client_count", &payload) {
            error!("Failed to emit client_count: {}", e);
        }
    });
}

pub fn on_join_conductor(
    socket: SocketRef,
    State(state): State<Arc<RwLock<AppState>>>,
) {
    info!("Conductor joined: {}", socket.id);
    socket.join("conductor");
    tokio::spawn(async move {
        let guard = state.read().await;
        let payload = ServerStatePayload {
            client_count: guard.connected_clients,
            midi_port: guard.midi_port.clone(),
            midi_connected: guard.midi_connected,
            midi_ports: guard.midi_ports.clone(),
            current_chord: guard.current_chord,
            chord_names: CHORDS_DATA.names.clone(),
            root_names: SCALES_DATA.root_names.clone(),
        };
        drop(guard);

        if let Err(e) = socket.emit("server_state", &payload) {
            error!("Failed to emit server_state: {}", e);
        }
    });
}

pub fn on_request_midi_ports(
    socket: SocketRef,
    State(state): State<Arc<RwLock<AppState>>>,
) {
    tokio::spawn(async move {
        let guard = state.read().await;
        let ports = guard.midi_ports.clone();
        let handle = guard.midi_handle.as_ref().cloned();
        drop(guard);

        if let Some(handle) = handle {
            if let Err(e) = handle.list_ports().await {
                error!("Failed to request MIDI port list: {}", e);
            }
        }

        let payload = MidiPortsPayload { ports };
        if let Err(e) = socket.emit("midi_ports", &payload) {
            error!("Failed to emit midi_ports: {}", e);
        }
    });
}

pub fn on_set_midi_port(
    socket: SocketRef,
    State(state): State<Arc<RwLock<AppState>>>,
    Data(data): Data<SetMidiPortData>,
) {
    let new_port = data.port;
    tokio::spawn(async move {
        let mut guard = state.write().await;
        guard.midi_port = new_port.clone();
        let connected = guard.midi_connected;
        let handle = guard.midi_handle.as_ref().cloned();
        guard.notify_gui(true);
        drop(guard);

        if let Some(handle) = handle {
            if let Err(e) = handle.reconnect(new_port.clone()).await {
                error!("Failed to send MIDI reconnect command: {}", e);
            }
        }

        let payload = MidiStatusPayload {
            connected,
            port: new_port,
        };
        if let Err(e) = socket.emit("midi_status", &payload) {
            error!("Failed to emit midi_status: {}", e);
        }
    });
}

pub fn on_set_chord(
    io: SocketIo,
    State(state): State<Arc<RwLock<AppState>>>,
    Data(data): Data<SetChordData>,
) {
    tokio::spawn(async move {
        queue_chord(data.root, data.chord_type, &io, &state).await;
    });
}

pub fn on_control_change(
    io: SocketIo,
    State(state): State<Arc<RwLock<AppState>>>,
    Data(data): Data<ControlChangeData>,
) {
    tokio::spawn(async move {
        queue_chord(data.control, data.value, &io, &state).await;
    });
}

pub async fn queue_chord(
    root: u8,
    chord_type: u8,
    io: &SocketIo,
    state: &Arc<RwLock<AppState>>,
) {
    let mut guard = state.write().await;
    if !guard.should_broadcast(root, chord_type) {
        return;
    }
    guard.current_chord = CurrentChord { root, chord_type };
    guard.notify_gui(true);
    drop(guard);

    info!("Broadcast chord: root={} type={}", root, chord_type);

    if let Err(e) = io.emit("control_change", &ControlChangePayload { control: root, value: chord_type }) {
        error!("Failed to emit control_change: {}", e);
    }
    if let Err(e) = io.to("conductor").emit("chord_changed", &ChordChangedPayload { root, chord_type }) {
        error!("Failed to emit chord_changed: {}", e);
    }
}

pub async fn handle_midi_output(
    output: MidiOutput,
    io: &SocketIo,
    state: &Arc<RwLock<AppState>>,
) {
    match output {
        MidiOutput::Chord(result) => {
            queue_chord(result.root, result.chord_type, io, state).await;
        }
        MidiOutput::ControlChange(control, value) => {
            queue_chord(control, value, io, state).await;
        }
        MidiOutput::Ports(ports) => {
            let mut guard = state.write().await;
            guard.midi_ports = ports.clone();
            guard.notify_gui(true);
            drop(guard);

            if let Err(e) = io.to("conductor").emit("midi_ports", &MidiPortsPayload { ports }) {
                error!("Failed to emit midi_ports: {}", e);
            }
        }
        MidiOutput::Status { connected, port } => {
            let mut guard = state.write().await;
            guard.midi_connected = connected;
            guard.midi_port = port.clone();
            guard.notify_gui(true);
            drop(guard);

            if let Err(e) = io.to("conductor").emit(
                "midi_status",
                &MidiStatusPayload {
                    connected,
                    port,
                },
            ) {
                error!("Failed to emit midi_status: {}", e);
            }
        }
        MidiOutput::ChannelChanged(ch) => {
            let mut guard = state.write().await;
            guard.midi_channel = ch;
            guard.notify_gui(true);
        }
    }
}

pub fn send_initial_gui_snapshot(
    state: &Arc<RwLock<AppState>>,
    gui_tx: &std::sync::mpsc::Sender<GuiStateSnapshot>,
) {
    let state = state.clone();
    let gui_tx = gui_tx.clone();
    tokio::spawn(async move {
        let guard = state.read().await;
        let snapshot = guard.snapshot(true);
        drop(guard);
        let _ = gui_tx.send(snapshot);
    });
}
