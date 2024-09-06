from flask import Flask, render_template
from HarmonyBridge import HarmonyBridge

app = Flask(__name__)
harmony_bridge = HarmonyBridge('CHORDION_MIDI Port 1', lambda control, value: print(f'Control: {control}, Value: {value}'))

def handle_control_change(control, value):
    print(f'Control: {control}, Value: {value}')


@app.route('/')
def index():
    return render_template('index.html')


if __name__ == '__main__':
    app.run(host='0.0.0.0', debug=True)
