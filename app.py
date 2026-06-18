import eventlet
eventlet.monkey_patch()

from flask import Flask, render_template
from flask_socketio import SocketIO
from HarmonyBridge import HarmonyBridge
from flask_cors import CORS
import json
import queue
import os
import time

app = Flask(__name__)

def _load_music_data():
    base = os.path.dirname(__file__)
    with open(os.path.join(base, 'static', 'data', 'chords.json')) as f:
        chords = json.load(f)
    with open(os.path.join(base, 'static', 'data', 'scales.json')) as f:
        scales = json.load(f)
    return {'chords': chords, 'scales': scales}

MUSIC_DATA = _load_music_data()
CORS(app, resources={r"/*": {"origins": "*"}})
# Configure SocketIO for better performance
socketio = SocketIO(
    app, 
    cors_allowed_origins="*", 
    async_mode='eventlet',
    ping_timeout=10,
    ping_interval=5,
    engineio_logger=True,  # Enable logging for debuggi ng
    logger=True            # Enable logging for debugging
)
harmony_bridge = None
event_queue = queue.Queue()

# Message deduplication cache
message_cache = {}
CACHE_TIMEOUT = 0.5  # seconds to consider a message as duplicate

@socketio.on('control_change')
def handle_control_change(control, value):
    try:
        # Create a unique key for this message
        message_key = f"{control}:{value}"
        current_time = time.time()
        
        # Check if this is a duplicate message
        if message_key in message_cache:
            last_time = message_cache[message_key]
            if current_time - last_time < CACHE_TIMEOUT:
                print(f"Ignoring duplicate message: {control}, {value}")
                return
        
        # Update the cache with the current time
        message_cache[message_key] = current_time
        
        # Clean up old cache entries
        for key in list(message_cache.keys()):
            if current_time - message_cache[key] > CACHE_TIMEOUT:
                del message_cache[key]
        
        # Add to the event queue
        event_queue.put((control, value))
        print(f"Queued message: {control}, {value}")
    except Exception as e:
        print(f'Error handling control change: {e}')

def process_event_queue():
    while True:
        if not event_queue.empty():
            control, value = event_queue.get()
            socketio.emit('control_change', {'control': control, 'value': value})
            print(f'->Emitted Control: {control}, Value: {value}')
        # Reduce sleep time for lower latency
        eventlet.sleep(0.01)

socketio.start_background_task(process_event_queue)

if os.environ.get('WERKZEUG_RUN_MAIN') == 'true':
    midi_port = os.environ.get('MIDI_PORT', 'Driver IAC Bus 1')
    midi_channel = int(os.environ.get('MIDI_CHANNEL', '0'))
    harmony_bridge = HarmonyBridge(midi_port, handle_control_change, midi_channel)

@socketio.on_error()
def error_handler(e):
    print(f"An error occurred: {e}")

@app.route('/')
def index():
    return render_template('index.html', music_data=MUSIC_DATA)

if __name__ == '__main__':
    import threading

    def run_harmony_bridge():
        print("Starting HarmonyBridge thread...")
        try:
            if harmony_bridge:
                ...
        except KeyboardInterrupt:
            harmony_bridge.close()

    harmony_bridge_thread = threading.Thread(target=run_harmony_bridge)
    harmony_bridge_thread.start()

    socketio.run(app, host='0.0.0.0', debug=True)

    harmony_bridge_thread.join()
