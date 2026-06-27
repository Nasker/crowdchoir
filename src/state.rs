use crate::chord_finder::ChordResult;
use crate::midi::MidiBridgeHandle;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

pub const CACHE_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct CurrentChord {
    pub root: u8,
    pub chord_type: u8,
}

impl From<ChordResult> for CurrentChord {
    fn from(result: ChordResult) -> Self {
        Self {
            root: result.root,
            chord_type: result.chord_type,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GuiStateSnapshot {
    pub connected_clients: usize,
    pub current_chord: CurrentChord,
    pub midi_port: String,
    pub midi_channel: u8,
    pub midi_connected: bool,
    pub midi_ports: Vec<String>,
    pub running: bool,
}

#[derive(Debug)]
pub struct AppState {
    pub connected_clients: usize,
    pub current_chord: CurrentChord,
    pub midi_port: String,
    pub midi_connected: bool,
    pub midi_ports: Vec<String>,
    pub midi_channel: u8,
    pub midi_handle: Option<MidiBridgeHandle>,
    pub gui_tx: Option<Sender<GuiStateSnapshot>>,
    pub message_cache: HashMap<(u8, u8), Instant>,
}

impl AppState {
    pub fn new(midi_port: String, midi_channel: u8) -> Self {
        Self {
            connected_clients: 0,
            current_chord: CurrentChord::default(),
            midi_port,
            midi_connected: false,
            midi_ports: Vec::new(),
            midi_channel,
            midi_handle: None,
            gui_tx: None,
            message_cache: HashMap::new(),
        }
    }

    pub fn snapshot(&self, running: bool) -> GuiStateSnapshot {
        GuiStateSnapshot {
            connected_clients: self.connected_clients,
            current_chord: self.current_chord,
            midi_port: self.midi_port.clone(),
            midi_channel: self.midi_channel,
            midi_connected: self.midi_connected,
            midi_ports: self.midi_ports.clone(),
            running,
        }
    }

    pub fn notify_gui(&self, running: bool) {
        if let Some(tx) = &self.gui_tx {
            let snapshot = self.snapshot(running);
            let _ = tx.send(snapshot);
        }
    }

    pub fn should_broadcast(&mut self, root: u8, chord_type: u8) -> bool {
        let now = Instant::now();
        let key = (root, chord_type);

        if let Some(&last_time) = self.message_cache.get(&key) {
            if now.duration_since(last_time) < CACHE_TIMEOUT {
                return false;
            }
        }

        self.message_cache.insert(key, now);
        self.message_cache.retain(|_, &mut t| now.duration_since(t) <= CACHE_TIMEOUT);
        true
    }
}
