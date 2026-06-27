# Rust Backend Migration Plan

Goal: replace the Python/Flask backend with a Rust backend plus a small native GUI control panel, while keeping the existing browser frontend untouched.

## Why this stack

- **Backend server**: `axum` + `socketioxide` (Socket.IO protocol compatible with current JS clients) + `tokio`
- **MIDI**: `midir` (cross-platform, replaces `mido`/`python-rtmidi`)
- **Static assets**: `rust-embed` so `templates/` and `static/` ship inside the binary
- **Native GUI**: `egui` + `eframe` — lightweight, single-binary, easy to start/stop the server from a window
- **Serialization**: `serde` + `serde_json`

The browser frontend (`templates/`, `static/`) is reused as-is. The Socket.IO event names and payloads stay the same.

---

## Project layout

```
crowdchoir/
├── Cargo.toml                 # workspace root
├── src/
│   ├── main.rs                # GUI entry point (eframe)
│   ├── server.rs              # axum + socketioxide server wiring
│   ├── state.rs               # shared app state (current chord, client count, MIDI status)
│   ├── midi.rs                # midir input + chord detection pipeline
│   ├── chord_finder.rs        # port of ChordFinder.py
│   ├── music_controller.rs    # port of MusicController.py / ChordMatrix / ScalesMatrix
│   └── events.rs              # Socket.IO event handlers
├── crowdchoir-server/
│   ├── Cargo.toml             # optional headless server crate
│   └── src/main.rs            # headless binary (no GUI, for deployment)
├── static/                    # existing browser assets
├── templates/                 # existing HTML templates
├── static/data/               # existing chords.json + scales.json
└── README.md / RUST_MIGRATION_PLAN.md
```

---

## Phase 1 — Rust project skeleton (RustRover setup)

1. Open the project in RustRover.
2. Install the Rust toolchain if needed:
   ```bash
   rustup default stable
   rustup target add x86_64-unknown-linux-gnu
   # for macOS deployment later: rustup target add x86_64-apple-darwin aarch64-apple-darwin
   ```
3. Create the workspace root:
   ```bash
   cargo init --name crowdchoir
   ```
4. Add workspace members to `Cargo.toml`:
   ```toml
   [workspace]
   members = [".", "crowdchoir-server"]
   resolver = "2"
   ```
5. Add dependencies to the root `Cargo.toml`:
   ```toml
   [dependencies]
   tokio = { version = "1.38", features = ["full"] }
   axum = { version = "0.7", features = ["tokio"] }
   socketioxide = { version = "0.15", features = ["state"] }
   tower = "0.4"
   tower-http = { version = "0.5", features = ["fs", "cors"] }
   midir = "0.10"
   eframe = { version = "0.28", features = ["default"] }
   egui = "0.28"
   rust-embed = "8.5"
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   anyhow = "1.0"
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   ```

6. In RustRover, run `Cargo` → `Reload Project` so the IDE resolves crates and imports.

---

## Phase 2 — Port the music theory modules

Files to port from Python to Rust:

- `ChordFinder.py` → `src/chord_finder.rs`
- `ChordMatrix.py` + `ScalesMatrix.py` + `MusicController.py` → `src/music_controller.rs`

### Porting notes

- Load `static/data/chords.json` and `static/data/scales.json` at compile time via `include_str!` or at runtime.
- Chord detection logic is the same: reduce notes to pitch classes, sort, rotate, compare against interval patterns.
- Write unit tests in RustRover with `#[cfg(test)]` and run them with the gutter icons or `cargo test`.

### Verification

- `cargo test` should pass all chord identification tests from the Python self-test suite.

---

## Phase 3 — MIDI input layer

Create `src/midi.rs`:

- Open `midir::MidiInput` and list available ports.
- Connect to a named port by substring or exact name.
- Maintain `played_notes: BTreeSet<u8>` for active MIDI note numbers.
- On `note_on`: insert note; when `len() >= 3`, arm a `tokio::time::sleep(Duration::from_millis(150))` debounce.
- On `note_off`: remove note and cancel the pending debounce timer.
- On `control_change`: forward directly to the broadcast callback.
- When debounce expires: call `chord_finder::identify_chord()` and emit a chord change to the server state channel.

### Porting notes from `HarmonyBridge.py`

- `midir` callbacks run on a dedicated thread, so use a `tokio::sync::mpsc` channel to send parsed MIDI events to the async server side.
- `reconnect(port)` should drop the old connection, create a new `MidiInput`, and reconnect.
- If connection fails, log the error and keep the server running (browser-only mode still works).

---

## Phase 4 — Socket.IO server

Create `src/server.rs` and `src/events.rs`:

### Static asset serving

- Use `rust-embed` to embed `static/` and `templates/`.
- Serve `/` → `index.html` with `music_data` replaced at runtime (or switch the frontend to fetch `/api/music_data` as JSON and remove Jinja2 injection).
- Serve `/conductor` → `conductor.html`.
- Serve everything else under `/static/` from embedded assets.

### Socket.IO events

Keep the same event names and payloads as the Python backend so the existing JS works unchanged:

| Event (server receives) | Handler action |
|---|---|
| `connect` | increment `connected_clients`, emit `client_count` to conductor room |
| `disconnect` | decrement `connected_clients`, emit `client_count` |
| `join_conductor` | join room `conductor`, emit `server_state` snapshot |
| `request_midi_ports` | emit `midi_ports` with `midir` port list |
| `set_midi_port` | reconnect MIDI, emit `midi_status` |
| `set_chord` | call broadcast pipeline |
| `control_change` | call broadcast pipeline (kept for compatibility) |

### State

Use `Arc<Mutex<AppState>>` or `Arc<RwLock<AppState>>` for:

```rust
struct AppState {
    connected_clients: usize,
    current_chord: Chord,
    midi_port: String,
    midi_connected: bool,
    midi_ports: Vec<String>,
}
```

Share the Socket.IO `SocketRef` handle with the MIDI layer so chord events can be emitted from the MIDI thread via a channel.

### Verification

- `cargo run` starts the server on `0.0.0.0:5000`.
- Open `http://localhost:5000` and `http://localhost:5000/conductor` in a browser.
- The conductor shows client count, MIDI status, and the chord picker works.
- MIDI chord changes (if a controller is available) broadcast to participants.

---

## Phase 5 — Native GUI control panel

Create `src/main.rs` as the GUI entry point using `eframe`:

### Window contents

- **Server status**: a large indicator (running / stopped) with port number.
- **Start/Stop button**: spawns or kills the `tokio` runtime running the server.
- **MIDI port selector**: dropdown of available ports, refresh button, connect button.
- **MIDI status dot**: green when connected, red otherwise.
- **Live info**: current chord, connected client count.
- **Log tail**: last 20 lines of `tracing` logs, useful for debugging at a venue.

### Implementation notes

- Run the server in a background thread with a `tokio::runtime::Runtime` created in `main()`.
- Use channels to send state updates from the server thread to the `egui` UI thread.
- Keep the GUI responsive: the server does not block the UI loop.
- On window close, cleanly shut down the tokio runtime and MIDI connection.

### Headless variant

Create `crowdchoir-server/src/main.rs` for a no-GUI server binary. This is what you deploy on a headless box or embed in a Docker image. It shares the same `server.rs` code via `lib.rs`.

---

## Phase 6 — Deployment and packaging

### Single binary

```bash
cargo build --release
```

Produces a single executable (with embedded assets) that can be copied to any machine.

### Cross-platform builds

```bash
# Linux
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# macOS (from Linux or macOS)
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Windows (from Linux)
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

For GUI builds, you may need platform-specific linker dependencies (e.g., `libgtk-3-dev` on Linux, `NSApplication` frameworks on macOS).

### Optional: Docker image

Use a multi-stage build to compile the `crowdchoir-server` headless binary and copy it into a tiny distroless image. This is useful for server-only deployments.

### TLS / HTTPS

Rust migration alone does not solve HTTPS. Keep the same solution as before: nginx reverse proxy, tunnelling service (ngrok, Cloudflare Tunnel), or RustRover deploy to a host with TLS termination.

---

## RustRover workflow tips

- Use `Code` → `Reformat Code` (`Ctrl+Alt+L`) to format with `rustfmt`.
- Use the `Cargo` tool window to run `cargo test`, `cargo run`, `cargo build`.
- Use `Alt+Enter` to auto-import traits and resolve missing dependencies.
- Add `#[derive(Debug)]` everywhere while debugging so you can `println!("{:?}", state)`.
- Keep the `tracing` subscriber initialized early in both `main.rs` and `crowdchoir-server/src/main.rs`.

---

## Verification checklist

- [x] `cargo test` passes all chord detection tests.
- [x] `cargo run` opens the GUI and the server starts on port 5000.
- [x] Browser participant page loads and connects via Socket.IO.
- [x] Conductor page loads, shows client count, and can refresh/change MIDI ports.
- [x] Manual chord picker in conductor broadcasts chord changes to participants.
- [x] MIDI controller input (if available) triggers chord detection and broadcasts.
- [x] Release binary runs on a clean machine without Rust or Python installed.
- [x] Headless `crowdchoir-server` binary can be deployed on a server without a display.

---

## Suggested order of work in RustRover

1. Set up the workspace and verify `cargo run` prints "hello crowdchoir".
2. Port `chord_finder.rs` and add tests.
3. Port `music_controller.rs` and add tests.
4. Build `midi.rs` to list ports and print note messages (no server yet).
5. Build `server.rs` with static asset serving and a simple `/api/music_data` JSON endpoint.
6. Add all Socket.IO event handlers and connect them to a stub state.
7. Wire the MIDI layer into the server so real chord changes broadcast.
8. Add the `eframe` GUI in `main.rs` to start/stop the server and show MIDI status.
9. Create the `crowdchoir-server` headless crate.
10. Build release binaries and test on a second machine.

---

## Open decisions to make before coding

1. **Music data injection**: keep the Jinja2-style `window.MUSIC_DATA` server-side render, or change `index.html` to fetch `/api/music_data`? The fetch approach is simpler in Rust.
2. **MIDI backend**: `midir` works for all platforms. Confirm it sees your target controller on Linux/macOS/Windows.
3. **GUI crate or single binary**: start with the single GUI binary; add the headless `crowdchoir-server` once the core is working.
4. **Scale and octave UI**: the existing `PLAN.md` items 3.1 and 3.2 can be done in the frontend in parallel or after the backend port.

---

## Estimated effort

- Project setup + first hello run: 1 hour
- Music theory port + tests: 3–4 hours
- MIDI layer: 2–3 hours
- Socket.IO server + static assets: 4–6 hours
- GUI control panel: 4–6 hours
- Headless server + packaging + deployment testing: 3–4 hours

Total: roughly **15–25 hours** for a solid, tested Rust rewrite with GUI and deployment binaries.
