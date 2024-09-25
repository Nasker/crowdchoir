import eventlet
eventlet.monkey_patch()

from flask import Flask, render_template
from flask_socketio import SocketIO
from HarmonyBridge import HarmonyBridge
from flask_cors import CORS
import queue
import os

app = Flask(__name__)
CORS(app, resources={r"/*": {"origins": "http://localhost:5000"}})
socketio = SocketIO(app, cors_allowed_origins="*", allow_credentials=True, async_mode='eventlet', broadcast=True)

event_queue = queue.Queue()

@socketio.on('control_change')
def handle_control_change(control, value):
    try:
        event_queue.put((control, value))
    except Exception as e:
        print(f"Error queueing control change: {e}")

def process_event_queue():
    while True:
        if not event_queue.empty():
            control, value = event_queue.get()
            socketio.emit('control_change', {'control': control, 'value': value})
            print(f'->Emitted Control: {control}, Value: {value}')
        eventlet.sleep(0.05)

socketio.start_background_task(process_event_queue)

if os.environ.get('WERKZEUG_RUN_MAIN') == 'true':
    # Only create the HarmonyBridge instance in the main process
    harmony_bridge = HarmonyBridge('CHORDION_MIDI Port 1', handle_control_change)

@socketio.on_error()
def error_handler(e):
    print(f"An error occurred: {e}")

@app.route('/')
def index():
    return render_template('index.html')

if __name__ == '__main__':
    import threading

    def run_harmony_bridge():
        print("Starting HarmonyBridge thread...")
        try:
            harmony_bridge
        except KeyboardInterrupt:
            harmony_bridge.close()

    harmony_bridge_thread = threading.Thread(target=run_harmony_bridge)
    harmony_bridge_thread.start()

    socketio.run(app, host='0.0.0.0', debug=True)

    harmony_bridge_thread.join()
