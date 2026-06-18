# Improvement Plan

This document proposes concrete improvements to CrowdChoir, ordered roughly by impact and effort. Each item is self-contained and can be tackled independently.

---

## Priority 1 — Fix Real Usability Blockers  ✅ Done

### 1.1 Make the MIDI port configurable without editing source  ✅

MIDI port and channel are now read from `MIDI_PORT` / `MIDI_CHANNEL` env vars. HarmonyBridge logs available ports on startup and continues in browser-only mode if the port is missing. The conductor page also supports switching ports live.

### 1.2 Detect 3-note triads in addition to 4-note chords  ✅

Replaced the fixed 4-note trigger with a 150 ms debounce timer. Any 3+ note combination fires chord detection after strumming settles.

### 1.3 Bundle dependencies (remove CDN requirement)  ✅

Tone.js and Socket.IO 4.3.2 are saved to `static/vendor/` and served locally. No internet required at the venue.

---

## Priority 2 — Eliminate Duplicated Code  ✅ Done

### 2.1 Single source of truth for music theory data (Option A)  ✅

`static/data/chords.json` and `scales.json` are the canonical source. Python matrix classes load from JSON at import time; Flask injects `window.MUSIC_DATA` into the participant page so JS reads the same data without a fetch.

---

## Priority 2b — Conductor Interface  ✅ Done

### 2b.1 Conductor dashboard at `/conductor`  ✅

A dark-themed dashboard at `http://<server>:5000/conductor` provides:
- Live **client count** (all connected browsers)
- **MIDI port status** indicator (green/red dot + port name)
- **Current chord** display (root note + chord type name)
- **MIDI port selector** with live Refresh and Connect buttons — switches the port without restarting the server
- **Manual chord picker** — 12 root note buttons × 16 chord type buttons; selecting both immediately broadcasts the chord to all participants (same pipeline as MIDI input)

---

## Priority 3 — UI / UX Improvements

### 3.1 Expose octave control

**Problem:** Octave is fixed at 3 (hardcoded in `main.js`). There is no way to play higher or lower.

**Fix:** Add +/- octave buttons to the UI. Range: 2–5.

**Effort:** 1 hour.

---

### 3.2 Expose scale selection

**Problem:** The scale system (14 scales) is fully implemented but not accessible from the UI. Only the default chromatic scale is ever used.

**Fix:** Add a scale picker (dropdown or pill buttons) to the header. Emit a `scale_change` Socket.IO event so the server and all clients stay in sync.

**Effort:** 2–3 hours.

---

### 3.3 Show which note each participant is playing

**Problem:** Participants have no visual feedback about what note they are triggering, or what chord is currently active beyond the text label.

**Fix:**
- Display a mini keyboard or staff graphic that highlights the active chord tones.
- Pulse the active XY-pad cell when a note plays.

**Effort:** 3–5 hours.

---

### 3.4 Add more sample sets

**Problem:** Only two instrument timbres exist (flute, ohs). Both have only 5 samples, leaving large gaps that Tone.js has to pitch-shift across.

**Fix:**
- Record or source additional samples (strings, pad, vibraphone).
- Increase sample density (every 3 semitones = much better quality).
- Consider using the Salamander Piano or other Creative Commons sample libraries.

**Effort:** varies (mostly asset work, minimal code change).

---

## Priority 4 — Robustness & Production Readiness

### 4.1 Production server setup

**Problem:** `python app.py` runs Flask's development server. It is single-threaded, restarts on errors, and not safe for public exposure.

**Fix:**
- Add a `Procfile` or `docker-compose.yml`.
- Use `gunicorn` with the eventlet worker: `gunicorn --worker-class eventlet -w 1 app:app`.
- Add a simple nginx reverse proxy for TLS termination (required for Web Audio on iOS Safari over HTTPS).

```
# Procfile
web: gunicorn --worker-class eventlet -w 1 --bind 0.0.0.0:$PORT app:app
```

**Effort:** 2–3 hours.

---

### 4.2 HTTPS / WSS support

**Problem:** Web Audio and device orientation APIs require a secure context (`https://`) on mobile browsers. The current plain HTTP setup breaks these features on iOS.

**Fix:** TLS termination via nginx + Let's Encrypt, or use a tunnelling service (ngrok, Cloudflare Tunnel) for workshop use.

**Effort:** 1–2 hours with a tunnelling service; 3–4 hours with full nginx + certbot setup.

---

### 4.3 Graceful MIDI reconnection

**Problem:** If the MIDI port disconnects (USB unplug, driver restart), `HarmonyBridge` stops working silently with no recovery path.

**Fix:**
- Wrap MIDI polling in a try/except loop.
- Attempt reconnect every 5 seconds, emitting a `midi_status` Socket.IO event so the UI can display a connection indicator.

**Effort:** 1–2 hours.

---

## Priority 5 — Feature Extensions

### 5.1 Chord input without a MIDI controller

**Problem:** Without a physical MIDI controller, the server just broadcasts nothing and the participants have no harmonic context.

**Fix:** Add a chord picker UI visible only to the "leader" (could be URL-based: `/?role=leader`). The leader selects root and chord type from dropdowns; the selection is broadcast to all clients exactly like a MIDI chord change.

**Effort:** 3–4 hours.

---

### 5.2 Record and playback

**Problem:** Sessions are ephemeral. There is no way to replay a performance or share it.

**Options:**
- **Server-side:** Log all `control_change` events with timestamps to a JSON file. Add a `/replay` endpoint that re-emits the log.
- **Client-side:** Record the Web Audio output using `MediaRecorder` API.

**Effort:** 3–5 hours for server-side replay.

---

### 5.3 Per-client volume and instrument

**Problem:** All participants use the same volume level. There is no way for an individual to adjust their own mix.

**Fix:** Local UI controls (volume slider, per-client instrument selector) that do not emit to the server. Purely client-side state.

**Effort:** 1–2 hours.

---

### 5.4 Conductor view

**Problem:** The person running the MIDI controller has no visual overview of how many participants are connected or what they are doing.

**Fix:** A `/conductor` route that shows a participant count, live chord display, and a simple chord-picker interface (feeds into 5.1 above).

**Effort:** 3–5 hours.

---

## Summary Table

| # | Item | Impact | Effort | Status |
|---|------|--------|--------|--------|
| 1.1 | Configurable MIDI port | High | Low | ✅ Done |
| 1.2 | 3-note triad detection | High | Low | ✅ Done |
| 1.3 | Bundle JS dependencies | High | Medium | ✅ Done |
| 2.1 | Single source for music data | Medium | Medium | ✅ Done |
| 2b.1 | Conductor dashboard | High | Medium | ✅ Done |
| 3.1 | Octave UI control | Medium | Low | Next |
| 3.2 | Scale selection UI (conductor + participant) | Medium | Medium | Next |
| 3.3 | Active note visualisation | Medium | Medium | Soon |
| 3.4 | More sample sets | High | Low (assets) | Soon |
| 4.1 | Production server | High | Medium | Before public deploy |
| 4.2 | HTTPS/WSS | High | Medium | Before public deploy |
| 4.3 | MIDI reconnection (auto-retry) | Medium | Low | Soon |
| 5.2 | Record + playback | Medium | Medium | Later |
| 5.3 | Per-client volume | Low | Low | Later |
