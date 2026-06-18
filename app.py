from flask import Flask, render_template
from flask_socketio import SocketIO, emit, join_room
from flask_cors import CORS
from HarmonyBridge import HarmonyBridge
import json
import mido
import queue
import os
import time

# ── App & config ────────────────────────────────────────────────────────────

app = Flask(__name__)
CORS(app, resources={r"/*": {"origins": "*"}})

socketio = SocketIO(
    app,
    cors_allowed_origins="*",
    async_mode='threading',
    ping_timeout=10,
    ping_interval=5,
)

def _load_music_data():
    base = os.path.dirname(__file__)
    with open(os.path.join(base, 'static', 'data', 'chords.json')) as f:
        chords = json.load(f)
    with open(os.path.join(base, 'static', 'data', 'scales.json')) as f:
        scales = json.load(f)
    return {'chords': chords, 'scales': scales}

MUSIC_DATA = _load_music_data()

# Ensure mido uses the rtmidi backend before any port queries
mido.set_backend('mido.backends.rtmidi')

# ── Server state ─────────────────────────────────────────────────────────────

midi_port    = os.environ.get('MIDI_PORT', 'Driver IAC Bus 1')
midi_channel = int(os.environ.get('MIDI_CHANNEL', '0'))

harmony_bridge    = None
event_queue       = queue.Queue()
message_cache     = {}
CACHE_TIMEOUT     = 0.5   # seconds

connected_clients = 0
current_chord     = {'root': None, 'chord_type': None}

# ── Core chord pipeline ───────────────────────────────────────────────────────

def _queue_chord(root, chord_type):
    """Dedup, persist state, queue for broadcast.  Called by MIDI input and conductor."""
    global current_chord
    message_key  = f"{root}:{chord_type}"
    current_time = time.time()

    if message_key in message_cache:
        if current_time - message_cache[message_key] < CACHE_TIMEOUT:
            return

    message_cache[message_key] = current_time
    for key in list(message_cache.keys()):
        if current_time - message_cache[key] > CACHE_TIMEOUT:
            del message_cache[key]

    current_chord = {'root': root, 'chord_type': chord_type}
    event_queue.put((root, chord_type))
    socketio.emit('chord_changed', {'root': root, 'chord_type': chord_type}, to='conductor')
    print(f"Queued chord: root={root} type={chord_type}")


def process_event_queue():
    while True:
        if not event_queue.empty():
            root, chord_type = event_queue.get()
            socketio.emit('control_change', {'control': root, 'value': chord_type})
            print(f"-> Broadcast control={root} value={chord_type}")
        socketio.sleep(0.01)

socketio.start_background_task(process_event_queue)

# ── Socket events: all clients ────────────────────────────────────────────────

@socketio.on('connect')
def handle_connect():
    global connected_clients
    connected_clients += 1
    socketio.emit('client_count', {'count': connected_clients}, to='conductor')

@socketio.on('disconnect')
def handle_disconnect():
    global connected_clients
    connected_clients = max(0, connected_clients - 1)
    socketio.emit('client_count', {'count': connected_clients}, to='conductor')

@socketio.on('control_change')
def handle_control_change(control, value):
    _queue_chord(control, value)

# ── Socket events: conductor page ─────────────────────────────────────────────

@socketio.on('join_conductor')
def handle_join_conductor():
    join_room('conductor')
    try:
        available_ports = mido.get_input_names()
    except Exception:
        available_ports = []
    emit('server_state', {
        'client_count':  connected_clients,
        'midi_port':     midi_port,
        'midi_connected': harmony_bridge is not None and harmony_bridge.port is not None,
        'midi_ports':    available_ports,
        'current_chord': current_chord,
        'chord_names':   MUSIC_DATA['chords']['names'],
        'root_names':    MUSIC_DATA['scales']['rootNames'],
    })

@socketio.on('request_midi_ports')
def handle_request_midi_ports():
    try:
        ports = mido.get_input_names()
    except Exception:
        ports = []
    emit('midi_ports', {'ports': ports})

@socketio.on('set_midi_port')
def handle_set_midi_port(data):
    global harmony_bridge, midi_port
    new_port = data.get('port', '')
    if harmony_bridge:
        success = harmony_bridge.reconnect(new_port)
    else:
        harmony_bridge = HarmonyBridge(new_port, _queue_chord, midi_channel)
        success = harmony_bridge.port is not None
    midi_port = new_port
    emit('midi_status', {'connected': success, 'port': midi_port})

@socketio.on('set_chord')
def handle_set_chord(data):
    root       = data.get('root')
    chord_type = data.get('chord_type')
    if root is not None and chord_type is not None:
        _queue_chord(root, chord_type)

# ── Error handler ─────────────────────────────────────────────────────────────

@socketio.on_error()
def error_handler(e):
    print(f"Socket error: {e}")

# ── Routes ────────────────────────────────────────────────────────────────────

@app.route('/')
def index():
    return render_template('index.html', music_data=MUSIC_DATA)

@app.route('/conductor')
def conductor():
    return render_template('conductor.html')

# ── Entry point ───────────────────────────────────────────────────────────────

if os.environ.get('WERKZEUG_RUN_MAIN') == 'true':
    harmony_bridge = HarmonyBridge(midi_port, _queue_chord, midi_channel)

if __name__ == '__main__':
    socketio.run(app, host='0.0.0.0', debug=True)
