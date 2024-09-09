import mido
import MusicController as mc


class HarmonyBridge:
    """
    A class that listens to a MIDI Virtual port and sends CommandControl structs to the callee using a callback.
    """

    def __init__(self, port_name, callback):
        """
        Initializes the HarmonyBridge object and sets up the MIDI input port with a callback.
        :param port_name: The name of the MIDI Virtual port to listen to.
        :param callback: The function to call when a Control Change is received.
        """
        mido.set_backend('mido.backends.rtmidi')
        print("Available MIDI input ports:", mido.get_input_names())
        self.port_name = port_name
        try:
            self.port = mido.open_input(self.port_name, callback=self.on_control_change)
        except IOError:
            print(f"Could not open MIDI input port: {self.port_name}")
            exit(1)
        self.callback = callback
        self.music_controller = mc.RTPMusicController()

    def on_control_change(self, message):
        """
        Callback function for the MIDI input port.
        :param message: The MIDI message received.
        """
        if message.type == 'control_change':
            self.music_controller.set_current_note(message.control)
            self.music_controller.set_current_chord(message.value)
            note = self.music_controller.get_current_root_note_name()
            chord = self.music_controller.get_chord_name()
            self.callback(note, chord)

    def close(self):
        """
        Closes the MIDI input port.
        """
        self.port.close()


if __name__ == '__main__':
    def callback(control, value):
        print(f'Control: {control}, Value: {value}')

    # Open a virtual port and start listening
    bridge = HarmonyBridge('CHORDION_MIDI Port 1', callback)

    try:
        input("Listening for MIDI control changes. Press Enter to exit...\n")
    finally:
        bridge.close()
