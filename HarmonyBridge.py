import mido


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
        self.port_name = port_name
        self.callback = callback
        self.port = mido.open_input(self.port_name, callback=self.on_control_change)

    def on_control_change(self, message):
        """
        Callback function for the MIDI input port.
        :param message: The MIDI message received.
        """
        if message.type == 'control_change':
            self.callback(message.control, message.value)

    def close(self):
        """
        Closes the MIDI input port.
        """
        self.port.close()


if __name__ == '__main__':
    def callback(control, value):
        print(f'Control: {control}, Value: {value}')


    mido.set_backend('mido.backends.rtmidi')
    # List all available ports
    print("Available MIDI input ports:", mido.get_input_names())

    # Open a virtual port and start listening
    bridge = HarmonyBridge('CHORDION_MIDI Port 1', callback)

    try:
        # Keep the script running to allow MIDI events to be processed
        input("Listening for MIDI control changes. Press Enter to exit...\n")
    finally:
        bridge.close()
