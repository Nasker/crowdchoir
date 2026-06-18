# Current Architecture

## Overview

CrowdChoir is a client-server application with a Python/Flask backend and a JavaScript/Web Audio frontend. The backend handles MIDI input and real-time WebSocket broadcasting; all audio synthesis runs entirely in the browser. There are two client types: **participants** (XY-pad players at `/`) and the **conductor** (dashboard at `/conductor`).

```
┌──────────────────────────────────────────────────────────────────┐
│                        HARDWARE LAYER                            │
│         MIDI Controller (Driver IAC Bus 1 / physical USB)        │
└───────────────────────────┬──────────────────────────────────────┘
                            │ MIDI messages (note_on / note_off / CC)
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│                        PYTHON BACKEND  (app.py)                  │
│                                                                  │
│  HarmonyBridge ──► ChordFinder ──► _queue_chord()               │
│                                        │                         │
│                    Conductor socket ───┘  event_queue            │
│                    events (set_chord,        │                   │
│                    set_midi_port, ...)        ▼                  │
│                                        process_event_queue()     │
│                                        socketio.emit(broadcast)  │
└──────────┬────────────────────────────────────┬──────────────────┘
           │ WebSocket /conductor room           │ WebSocket all
           ▼                                     ▼
┌──────────────────────────┐     ┌───────────────────────────────────┐
│  CONDUCTOR  /conductor   │     │  PARTICIPANTS  /  (n browsers)    │
│                          │     │                                   │
│  Dashboard:              │     │  WebSocketHandler                 │
│  • client count          │     │  ──► RTPMusicController           │
│  • MIDI status           │     │  ──► Synth (Tone.js)              │
│  • current chord         │     │        Audio chain → Speakers     │
│  • MIDI port selector    │     │  XY-pad (touch / mouse)           │
│  • chord picker          │     └───────────────────────────────────┘
└──────────────────────────┘
```

---

## Backend Components

### [app.py](app.py)

The entry point and orchestrator.

- Creates the Flask app and Socket.IO server (eventlet async mode).
- CORS enabled for all origins (development-friendly).
- `_queue_chord(root, chord_type)` is the single funnel for all chord events — called by the MIDI thread and by the conductor socket. It deduplicates, updates `current_chord` state, and pushes to `event_queue`. It also emits `chord_changed` directly to the `'conductor'` Socket.IO room.
- A background greenlet drains the queue and broadcasts `control_change` to all clients.
- Tracks `connected_clients` via `connect` / `disconnect` events; emits `client_count` updates to the conductor room.

**Conductor socket events:**

| Event (server receives) | Action |
|---|---|
| `join_conductor` | client joins `'conductor'` room; receives `server_state` snapshot |
| `request_midi_ports` | returns `midi_ports` list |
| `set_midi_port` | calls `HarmonyBridge.reconnect()`; emits `midi_status` back |
| `set_chord` | calls `_queue_chord()` directly |

**Key settings:**
| Setting | Value |
|---|---|
| Listen address | 0.0.0.0:5000 |
| Async mode | eventlet |
| Ping timeout | 10 s |
| Ping interval | 5 s |
| Dedup window | 0.5 s |

### [HarmonyBridge.py](HarmonyBridge.py)

MIDI listener that bridges hardware to the chord detection pipeline.

- Opens a named MIDI input port via `mido` (port name from `MIDI_PORT` env var; logs available ports on startup).
- Tracks active notes in `played_notes`.
- On `note_on`: adds note (deduped); when ≥ 3 notes are held, arms a 150 ms debounce timer via `_schedule_detect()`.
- On `note_off`: removes note and cancels the pending timer so mid-change states don't fire.
- On `control_change`: deduplicates and passes directly to callback.
- `detect_chord()` fires when the debounce timer expires; calls `ChordFinder.identify_chord()` and invokes the callback.
- `reconnect(new_port)`: closes the current port and opens a new one — used by the conductor's `set_midi_port` event. Returns `True`/`False`.
- If the port cannot be opened, logs a warning with available ports and continues (browser-only mode works).

### [ChordFinder.py](ChordFinder.py)

Pure function: MIDI note array → (root_note: int 0-11, chord_type: int 0-15).

Algorithm:
1. Reduce all notes to pitch classes (mod 12).
2. Sort and de-duplicate.
3. For each rotation of the pitch-class set, compute intervals relative to the first note.
4. Compare against each chord pattern in `ChordMatrix`.
5. Return the first match; return `(None, None)` if no match found.

Includes a comprehensive self-test suite covering major, minor, seventh, diminished, augmented, suspended, and inverted chords.

### [MusicController.py](MusicController.py) / [ChordMatrix.py](ChordMatrix.py) / [ScalesMatrix.py](ScalesMatrix.py)

Server-side harmonic state and music theory data.

- **MusicController**: holds current `root_note`, `chord_type`, `scale`, `octave`, `velocity`, `channel`. Provides `get_midi_note(step)` to calculate an absolute MIDI note number.
- **ChordMatrix**: 16 chord definitions as semitone interval arrays (both chord-step and arpeggio variants).
- **ScalesMatrix**: 14 scale definitions including Western modes, blues, Japanese, Hawaiian, and a MIDI drum mapping.

> These are duplicated verbatim in JavaScript (`RTPMusicController.js`, `RTPChordMatrix.js`, `RTPScaleMatrix.js`). Any change to the music theory data must be applied in both places.

---

## Frontend Components

### [templates/conductor.html](templates/conductor.html)

Self-contained conductor dashboard (no module imports — plain `<script>`).

- Connects to Socket.IO and immediately emits `join_conductor` to join the server-side `'conductor'` room.
- On `server_state`: populates root/chord name arrays, builds the chord picker grid, sets initial UI state.
- Live updates via `client_count`, `midi_status`, `chord_changed` events.
- **MIDI selector**: dropdown of available ports populated from `server_state.midi_ports`; Refresh re-queries via `request_midi_ports`; Connect emits `set_midi_port`.
- **Chord picker**: 12 root note buttons + 16 chord type buttons. Selecting both root and type immediately emits `set_chord` — no separate Submit button needed. Incoming `chord_changed` events (from MIDI hardware) keep the picker highlight in sync.

### [templates/index.html](templates/index.html)

Participant SPA shell. Loads Tone.js and Socket.IO from local `static/vendor/`, injects `window.MUSIC_DATA` from Flask, then imports `main.js` as an ES6 module. Defines the DOM structure: header, XY-pad div, sample selector buttons.

### [static/main.js](static/main.js)

Frontend entry point. Instantiates all frontend classes, wires events, and coordinates:

- XY-pad mouse/touch → calls `synth.setFilter(x, y)` and `synth.playNote(step)`.
- Sample selector buttons → calls `synth.loadSamples(name)`.
- WebSocket `control_change` event → updates `RTPMusicController` and re-displays the chord name.

### [static/Synth.js](static/Synth.js)

Web Audio synthesis engine built on Tone.js.

**Signal chain:**
```
Sampler → BiquadFilter (lowpass) → AmplitudeEnvelope → Compressor → FeedbackDelay → Limiter → Destination
```

- **Sampler**: loads 5 MP3 notes per instrument set (C3, E3, G3, A3, C4). Tone.js automatically picks the nearest sample and pitch-shifts it.
- **Filter**: cutoff mapped exponentially from Y-axis (80 Hz–12 kHz); Q (resonance) mapped from X-axis (0.1–10).
- **Envelope**: short attack (0.01 s), medium decay/release.
- **Compressor + Limiter**: prevents clipping in dense chord scenarios.
- **FeedbackDelay**: adds ambience; wet mix ~0.2.

### [static/RTPMusicController.js](static/RTPMusicController.js)

JavaScript mirror of `MusicController.py`. Receives `(root_note, chord_type)` from the WebSocket and calculates which MIDI note to play for a given chord step and octave.

### [static/WebSocketHandler.js](static/WebSocketHandler.js)

Thin Socket.IO client wrapper. Connects to `ws://<hostname>:5000`, listens for `control_change`, fires a callback to `main.js`.

### [static/UserInteraction.js](static/UserInteraction.js)

Stub for additional input modalities (device orientation / gyroscope). Currently incomplete.

### [static/styles.css](static/styles.css)

Dark theme. CSS custom properties for colours. Flexbox layout. Responsive breakpoints for mobile. Glassmorphism-style panels.

---

## Data Flow

### MIDI chord → browser

```
Controller plays 4 notes
  └─► HarmonyBridge.on_note_on()    # buffers notes
        └─► ChordFinder.identify_chord()
              └─► callback(root, chord_type)
                    └─► event_queue.put(...)
                          └─► background greenlet
                                └─► socketio.emit('control_change', {control: root, value: chord_type})
                                      └─► [all browsers] WebSocketHandler.onControlChange()
                                            └─► RTPMusicController.setChord(root, chord_type)
```

### User touch → audio

```
User touches XY-pad at (x, y)
  └─► main.js mousedown/touchstart handler
        ├─► synth.setFilter(x, y)           # updates Tone.js filter in real time
        └─► synth.playNote(chordStep)
              └─► RTPMusicController.getMidiNote(step)   # root + chord + octave
                    └─► Sampler.triggerAttack(note)
                          └─► Filter → Envelope → Compressor → Delay → Limiter → Speakers
```

---

## Deployment

Currently runs as a Flask development server (`python app.py`). Not configured for production deployment (no WSGI server, no process manager, no TLS).

For local network use (e.g. a performance/workshop), this is sufficient provided the machine running the server and the participants are on the same WiFi network.

---

## Known Limitations

1. **No HTTPS/WSS** — WebSocket runs over plain HTTP; required for Web Audio on some mobile browsers in production contexts.
2. **No client identity** — All participants are anonymous; there is no way to differentiate or address individual users.
3. **`UserInteraction.js` is incomplete** — Device orientation / gyroscope input is scaffolded but not functional.
4. **No scale selection in the UI** — The scale system is implemented in the music controller but not exposed to participants or the conductor.
5. **Fixed octave in frontend** — Octave is initialised to 3 and not adjustable from the UI.
6. **Development server only** — No production WSGI config, process manager, or TLS termination.
7. **Conductor client count includes itself** — The conductor browser counts as one of the connected clients in the dashboard.
