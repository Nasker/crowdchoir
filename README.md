# CrowdChoir

A collaborative, web-based music application that lets multiple people play harmonies together in real time. A MIDI controller connected to the server drives the harmonic context — every browser that opens the page hears the same chords and can play notes within that harmony using an XY-pad.

## How It Works

1. A musician plays chords on a connected MIDI controller **or** picks a chord on the Conductor page.
2. The server detects the chord (root note + chord type) and broadcasts it over WebSocket to every connected browser.
3. Each browser participant uses the XY-pad to play notes — the X-axis selects which chord step to play, the Y-axis controls filter cutoff, and horizontal drag controls filter resonance.
4. All audio synthesis happens in the browser using the Web Audio API (via Tone.js) and a set of sampled instruments.

The result is a guided collective improvisation: the leader's hands shape what key everyone is playing in, while each participant expresses themselves within that harmonic space.

## Features

- Real-time chord detection from MIDI input (3+ note debounce detection)
- **Conductor page** (`/conductor`) — MIDI port selector, live client count, current chord display, manual chord picker
- WebSocket broadcast to unlimited concurrent browser clients
- XY-pad interface (touch and mouse) for each participant
- Two sampled instrument sets: Mello Flute and Mello Ohs
- Audio effects chain: filter → envelope → compression → feedback delay → limiter
- 16 chord types and 14 scales (single JSON source of truth for Python and JS)
- Responsive design with mobile touch support

## Requirements

- Python 3.9+
- A MIDI controller or virtual MIDI port
- Modern browser with Web Audio API support (Chrome, Firefox, Safari, Edge)

## Setup & Running

```bash
# Install Python dependencies
pip install -r requirements.txt

# Start the server
python app.py
```

The server starts on `http://0.0.0.0:5000`. Open that address in any browser on the same network.

### MIDI Configuration

The server defaults to listening on the `Driver IAC Bus 1` virtual MIDI port (macOS). To use a different port, edit the `midi_port` variable in [app.py](app.py) and [HarmonyBridge.py](HarmonyBridge.py).

To list available MIDI ports on your machine:

```python
import mido
print(mido.get_input_names())
print(mido.get_output_names())
```

## Project Structure

```
crowdchoir/
├── app.py                     # Flask server + Socket.IO + MIDI entry point
├── HarmonyBridge.py           # MIDI listener, chord detection, port management
├── ChordFinder.py             # Identifies chord type from a set of MIDI notes
├── MusicController.py         # Server-side harmonic state manager
├── ChordMatrix.py             # Loads chord data from JSON
├── ScalesMatrix.py            # Loads scale data from JSON
├── requirements.txt           # Python dependencies
├── templates/
│   ├── index.html             # Participant page (XY-pad)
│   └── conductor.html         # Conductor dashboard
└── static/
    ├── main.js                # Participant frontend entry point
    ├── Synth.js               # Web Audio synthesis engine (Tone.js)
    ├── RTPMusicController.js  # Client-side harmonic state
    ├── RTPChordMatrix.js      # Client chord data (from window.MUSIC_DATA)
    ├── RTPScaleMatrix.js      # Client scale data (from window.MUSIC_DATA)
    ├── WebSocketHandler.js    # Socket.IO client wrapper
    ├── UserInteraction.js     # Input handling
    ├── styles.css             # Dark theme UI
    ├── data/
    │   ├── chords.json        # Single source of truth for chord definitions
    │   └── scales.json        # Single source of truth for scale definitions
    ├── vendor/
    │   ├── tone.js            # Tone.js (local copy, no CDN required)
    │   └── socket.io.min.js   # Socket.IO client (local copy)
    └── samples/               # MP3 audio samples
        ├── mello_flute/       # C3 E3 G3 A3 C4
        └── mello_ohs/         # C3 E3 G3 A3 C4
```

## Usage

### Participants
1. Open `http://<server-ip>:5000` in a browser on the same network.
2. Click **Start Audio** to initialize the Web Audio context (required by browsers).
3. Touch or click anywhere on the XY-pad to play a note:
   - **Y-axis**: filter cutoff frequency (bottom = dark/muffled, top = bright/open)
   - **X-axis**: selects chord step and filter resonance
4. Use the **Mello Flute / Mello Ohs** buttons to switch instruments.

### Conductor
1. Open `http://<server-ip>:5000/conductor` in a browser.
2. The dashboard shows the number of connected browsers, MIDI port status, and current chord.
3. Select a MIDI input port from the dropdown and click **Connect** to switch ports live.
4. Click **Refresh** to re-scan available MIDI ports.
5. Use the **Root** and **Type** buttons to set the chord manually — it broadcasts to all participants immediately.
6. MIDI controller input and manual chord picks both go through the same broadcast pipeline.

## Dependencies

| Package | Version | Purpose |
|---|---|---|
| Flask | 3.0.0 | Web server |
| Flask-SocketIO | 5.3.6 | WebSocket integration (threading async mode) |
| mido | 1.2.10 | MIDI I/O |
| python-rtmidi | 2.5.0 | Real-time MIDI backend |
| Tone.js | (CDN) | Web Audio synthesis |
| Socket.IO | 4.3.2 (CDN) | WebSocket client |
