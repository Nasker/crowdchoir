import mido
import time
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
        self.DEBOUNCE_DELAY = 0.15  # seconds — wait for strumming to settle

        # Debounce timer for chord detection
        self._chord_timer = None
        self._timer_lock = threading.Lock()

        # Deduplication cache
        self.last_message = None
        self.last_message_time = 0
        self.dedup_timeout = 0.5  # seconds
        self.port = None
        try:
            self.port = mido.open_input(self.port_name, callback=self.select_message_type)
            print(f"Opened MIDI input port: {self.port_name}")
        except IOError:
            print(f"WARNING: Could not open MIDI input port '{self.port_name}'. "
                  f"Set the MIDI_PORT env var to one of: {mido.get_input_names()}")
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
        # Create a unique message identifier
        current_message = (message.control, message.value)
        current_time = time.time()
        
        # Check if this is a duplicate message
        if self.last_message == current_message and \
           (current_time - self.last_message_time) < self.dedup_timeout:
            print(f"Ignoring duplicate MIDI message: {message.control}, {message.value}")
            return
        
        # Update the last message info
        self.last_message = current_message
        self.last_message_time = current_time
        
        print(f"Control Change: {message.control} Value: {message.value}")
        self.callback(message.control, message.value)

    def _schedule_detect(self):
        """Cancel any pending chord detection and reschedule after DEBOUNCE_DELAY."""
        with self._timer_lock:
            if self._chord_timer is not None:
                self._chord_timer.cancel()
            self._chord_timer = threading.Timer(self.DEBOUNCE_DELAY, self.detect_chord)
            self._chord_timer.daemon = True
            self._chord_timer.start()

    def on_note_on(self, message):
        """
        Callback function for the MIDI input port.
        :param message: The MIDI message received.
        """
        if message.channel == self.channel:
            if message.note not in self.played_notes:
                self.played_notes.append(message.note)
            if len(self.played_notes) >= 3:
                self._schedule_detect()

    def detect_chord(self):
        """
        Called after the timeout period to detect the chord.
        """
        if len(self.played_notes) > 0:
            control, value = self.chord_finder.identify_chord(self.played_notes)
            self.callback(control, value)
            # Comment out MIDI output to prevent feedback loops
            # self.outport.send(mido.Message('control_change', control=control, value=value))
            # print(f'SEND CONTROL TO BUIT_MIDI: {control}, {value}')
        self.played_notes.clear()

    def on_note_off(self, message):
        """
        Callback function for the MIDI input port.
        :param message: The MIDI message received.
        """
        if message.channel == self.channel:
            if message.note in self.played_notes:
                self.played_notes.remove(message.note)
            # Cancel pending detection — chord is changing
            with self._timer_lock:
                if self._chord_timer is not None:
                    self._chord_timer.cancel()
                    self._chord_timer = None

    def close(self):
        """
        Closes the MIDI input port.
        """
        if self.port:
            self.port.close()


if __name__ == '__main__':
    def callback(control, value):
        print(f'Control: {control}, Value: {value}')

    # Open a virtual port and start listening
    # bridge = HarmonyBridge('Driver IAC Bus 1', callback, 0)
    # bridge = HarmonyBridge('UMX 25', callback)
    bridge = HarmonyBridge('CHORDION_MIDI Port 1', callback, 3)
    # create an instance of mido as an output to BUIT_MIDI
    try:
        input("Listening for MIDI control changes. Press Enter to exit...\n")
    finally:
        bridge.close()
