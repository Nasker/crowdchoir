import eventlet
eventlet.monkey_patch()

from flask import Flask, render_template
from flask_socketio import SocketIO
from HarmonyBridge import HarmonyBridge
from flask_cors import CORS

app = Flask(__name__)
CORS(app, resources={r"/*": {"origins": "http://localhost:5000"}})
socketio = SocketIO(app, cors_allowed_origins="*",  allow_credentials=True, async_mode='eventlet')

@socketio.on('control_change')
def handle_control_change(control, value):
    try:
        socketio.emit('control_change', {'control': control, 'value': value})
        print(f'->Control: {control}, Value: {value}')
    except Exception as e:
        print(f"Error sending control change: {e}")

@socketio.on('test_event')
def test_event():
    socketio.emit('control_change', {'control': 'test', 'value': 100})
    print('->Test event emitted')

harmony_bridge = HarmonyBridge('CHORDION_MIDI Port 1', handle_control_change)
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
