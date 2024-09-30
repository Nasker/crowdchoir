import mido
import threading
from ChordFinder import ChordFinder
from MusicController import RTPMusicController
class HarmonyBridge:
    """
    A class that listens to a MIDI Virtual port and sends CommandControl structs to the callee using a callback.
    """
    instance_count = 0

    def __init__(self, port_name, callback, channel=0):
        """
        Initializes the HarmonyBridge object and sets up the MIDI input port with a callback.
        :param port_name: The name of the MIDI Virtual port to listen to.
        :param callback: The function to call when a Control Change is received.
        """
        mido.set_backend('mido.backends.rtmidi')
        print("Available MIDI input ports:", mido.get_input_names())
        self.port_name = port_name
        self.played_notes = []
        self.channel = channel
        self.timeout_duration = 0.03
        self.detection_timer = None
        try:
            self.port = mido.open_input(self.port_name, callback=self.select_message_type)
        except IOError:
            print(f"Could not open MIDI input port: {self.port_name}")
            exit(1)
        self.callback = callback
        HarmonyBridge.instance_count += 1  # Increment the instance count when a new instance is created
        print(f"Number of HarmonyBridge instances: {HarmonyBridge.instance_count}")
        self.music_controller = RTPMusicController()
        self.chord_finder = ChordFinder(self.music_controller)

    def select_message_type(self, message):
        """
        Selects the appropriate callback function based on the MIDI message type.
        :param message: The MIDI message received.
        """
        if message.type == 'control_change':
            self.on_control_change(message)
        elif message.type == 'note_on':
            self.on_note_on(message)
        elif message.type == 'note_off':
            self.on_note_off(message)

    def on_control_change(self, message):
        """
        Callback function for the MIDI input port.
        :param message: The MIDI message received.
        """
        print(f"Control Change: {message.control} Value: {message.value}")
        self.callback(message.control, message.value)

    def on_note_on(self, message):
        """
        Callback function for the MIDI input port.
        :param message: The MIDI message received.
        """
        if message.channel == self.channel:
            self.played_notes.append(message.note)
            if self.detection_timer is not None:
                self.detection_timer.cancel()
            self.detection_timer = threading.Timer(self.timeout_duration, self.detect_chord)
            self.detection_timer.start()

    def detect_chord(self):
        """
        Called after the timeout period to detect the chord.
        """
        if len(self.played_notes) > 0:
            control, value = self.chord_finder.identify_chord(self.played_notes)
            self.callback(control, value)
        self.played_notes.clear()

    def on_note_off(self, message):
        """
        Callback function for the MIDI input port.
        :param message: The MIDI message received.
        """
        if message.channel == self.channel:
            self.played_notes.remove(message.note)

    def close(self):
        """
        Closes the MIDI input port.
        """
        self.port.close()


if __name__ == '__main__':
    def callback(control, value):
        print(f'Control: {control}, Value: {value}')

    # Open a virtual port and start listening
    # bridge = HarmonyBridge('Driver IAC Bus 1', callback, 0)
    # bridge = HarmonyBridge('UMX 25', callback)
    bridge = HarmonyBridge('CHORDION_MIDI Port 1', callback, 3)

    try:
        input("Listening for MIDI control changes. Press Enter to exit...\n")
    finally:
        bridge.close()
