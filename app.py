from flask import Flask, render_template
from HarmonyBridge import HarmonyBridge

def handle_control_change(control, value):
    print(f'Control: {control}, Value: {value}')

app = Flask(__name__)
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

    app.run(host='0.0.0.0', debug=True)

    harmony_bridge_thread.join()
