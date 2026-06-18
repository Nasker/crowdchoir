# Current Architecture

## Overview

CrowdChoir is a client-server application with a Python/Flask backend and a JavaScript/Web Audio frontend. The backend handles MIDI input and real-time WebSocket broadcasting; all audio synthesis runs entirely in the browser.

```
┌──────────────────────────────────────────────────────────────────┐
│                        HARDWARE LAYER                            │
│         MIDI Controller (Driver IAC Bus 1 / physical USB)        │
└───────────────────────────┬──────────────────────────────────────┘
                            │ MIDI messages (note_on / note_off / CC)
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│                        PYTHON BACKEND                            │
│                                                                  │
│  HarmonyBridge ──► ChordFinder ──► app.py (Flask + Socket.IO)   │
│  MusicController    ChordMatrix     Event Queue + Dedup          │
│  ScalesMatrix                                                    │
└───────────────────────────┬──────────────────────────────────────┘
                            │ WebSocket (Socket.IO)
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│             JAVASCRIPT FRONTEND  (n concurrent browsers)         │
│                                                                  │
│  WebSocketHandler ──► RTPMusicController ──► Synth (Tone.js)    │
│  UserInteraction        RTPChordMatrix         Audio chain       │
│  (XY-pad / touch)       RTPScaleMatrix         Speakers          │
└──────────────────────────────────────────────────────────────────┘
```

---

## Backend Components

### [app.py](app.py)

The entry point and orchestrator.

- Creates the Flask app and Socket.IO server (eventlet async mode).
- CORS enabled for all origins (development-friendly).
- On startup, initialises `HarmonyBridge` with a callback that pushes chord changes into a thread-safe `event_queue`.
- A background greenlet drains the queue and calls `socketio.emit('control_change', ...)` to all connected clients.
- Message deduplication: ignores identical `(control, value)` pairs that arrive within 0.5 s of each other.
- Single route `/` serves `index.html`.

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

- Opens a named MIDI input port via `mido`.
- Tracks active notes in `played_notes` (a list).
- On `note_on`: appends the note; when the buffer reaches `n_notes_detection` (4), triggers chord detection.
- On `note_off`: removes the note from the buffer.
- On `control_change`: passes directly to the callback.
- Deduplication: ignores the same event within 0.5 s.
- Calls `ChordFinder.identify_chord()` and invokes the registered callback with `(root_note, chord_type)`.

**Limitation:** Port name is hardcoded (`Driver IAC Bus 1`). The port must exist or the bridge silently fails to connect.

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

### [templates/index.html](templates/index.html)

Minimal SPA shell. Loads Tone.js and Socket.IO from CDN, then imports `main.js` as an ES6 module. Defines the DOM structure: header, XY-pad div, sample selector buttons.

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

1. **Hardcoded MIDI port** — `HarmonyBridge.py` hardcodes `Driver IAC Bus 1`. Changing the port requires editing source code.
2. **Duplicated music theory data** — `ChordMatrix`, `ScalesMatrix`, and `MusicController` exist in both Python and JavaScript with no single source of truth.
3. **4-note detection only** — The chord trigger fires exactly when 4 notes are active. 3-note triads are not detected.
4. **CDN dependencies** — Tone.js and Socket.IO are loaded from public CDNs; the app breaks without internet access.
5. **No HTTPS/WSS** — WebSocket runs over plain HTTP; required for Web Audio on some mobile browsers in production contexts.
6. **No client identity** — All participants are anonymous; there is no way to differentiate or address individual users.
7. **`UserInteraction.js` is incomplete** — Device orientation / gyroscope input is scaffolded but not functional.
8. **No scale selection in the UI** — The scale system is implemented in the music controller but not exposed to users.
9. **Fixed octave in frontend** — Octave is initialised to 3 and not adjustable from the UI.
10. **Development server only** — No production WSGI config, process manager, or TLS termination.
