from MusicController import RTPMusicController
from MusicController import chord_name
from MusicController import root_name

class ChordFinder:
    """
    A class to identify chords based on MIDI notes.
    Uses RTPMusicController to access predefined chord structures and names.
    """

    def __init__(self, music_controller):
        self.music_controller = music_controller

    def midi_to_pitch_classes(self, midi_notes):
        """Convert MIDI notes to pitch classes."""
        return sorted([note % 12 for note in midi_notes])

    def get_intervals(self, notes):
        """Get the intervals between the notes."""
        return [(notes[i + 1] - notes[i]) % 12 for i in range(len(notes) - 1)]

    def identify_chord(self, midi_notes):
        """Identify the chord by comparing it with the music controller's chord patterns."""
        pitch_classes = self.midi_to_pitch_classes(midi_notes)
        for i in range(len(pitch_classes)):
            rotated_notes = pitch_classes[i:] + pitch_classes[:i]
            intervals = self.get_intervals(rotated_notes)
            for chord_type in range(len(chord_name)):
                self.music_controller.set_current_chord(chord_type)
                steps = [self.music_controller.chords.getChordStep(step) for step in
                         range(self.music_controller.chords.getChordSteps())]
                chord_intervals = [(steps[i + 1] - steps[i]) % 12 for i in range(len(steps) - 1)]
                if intervals[:len(chord_intervals)] == chord_intervals[:len(intervals)]:
                    root_note = rotated_notes[0]
                    chord_name_str = self.music_controller.get_chord_name()
                    root_name_str = root_name[root_note]
                    print(f"Root: {root_name_str}, Chord: {chord_name_str}")
                    return root_note, chord_type
        print("Chord not found")
        return 0, 0


# Example usage
if __name__ == "__main__":
    test_chords = [
        # Basic Triads and Inversions
        ([60, 64, 67], "C Major"),  # C-E-G
        ([64, 67, 72], "C Major 1st Inversion"),  # E-G-C
        ([67, 72, 76], "C Major 2nd Inversion"),  # G-C-E
        ([57, 60, 64], "A Minor"),  # A-C-E
        ([60, 64, 69], "A Minor 1st Inversion"),  # C-E-A
        ([64, 69, 72], "A Minor 2nd Inversion"),  # E-A-C

        # Seventh Chords
        ([55, 59, 62, 65], "G Dominant 7th"),  # G-B-D-F
        ([59, 62, 65, 67], "G Dominant 7th 1st Inversion"),  # B-D-F-G
        ([62, 65, 67, 71], "G Dominant 7th 2nd Inversion"),  # D-F-G-B
        ([65, 67, 71, 74], "G Dominant 7th 3rd Inversion"),  # F-G-B-D

        # Diminished Chords and Inversions
        ([62, 65, 68], "D Diminished"),  # D-F-Ab
        ([65, 68, 74], "D Diminished 1st Inversion"),  # F-Ab-D
        ([68, 74, 77], "D Diminished 2nd Inversion"),  # Ab-D-F

        # Augmented Chords
        ([64, 68, 72], "E Augmented"),  # E-G#-C
        ([68, 72, 76], "E Augmented 1st Inversion"),  # G#-C-E
        ([72, 76, 80], "E Augmented 2nd Inversion"),  # C-E-G#

        # Complex Seventh Chords and Inversions
        ([65, 69, 72, 76], "F Major 7th"),  # F-A-C-E
        ([69, 72, 76, 77], "F Major 7th 1st Inversion"),  # A-C-E-F
        ([72, 76, 77, 81], "F Major 7th 2nd Inversion"),  # C-E-F-A
        ([76, 77, 81, 84], "F Major 7th 3rd Inversion"),  # E-F-A-C

        # Suspended Chords
        ([60, 65, 67], "C Suspended 4th"),  # C-F-G
        ([65, 67, 72], "C Suspended 4th 1st Inversion"),  # F-G-C

        # Diminished Seventh Chords and Inversions
        ([59, 62, 65, 68], "B Diminished 7th"),  # B-D-F-Ab
        ([62, 65, 68, 71], "B Diminished 7th 1st Inversion"),  # D-F-Ab-B
        ([65, 68, 71, 74], "B Diminished 7th 2nd Inversion"),  # F-Ab-B-D
        ([68, 71, 74, 77], "B Diminished 7th 3rd Inversion"),  # Ab-B-D-F

        # Extended Chords
        ([60, 64, 67, 71, 74], "C Major 9th"),  # C-E-G-D
        ([62, 65, 69, 72, 76], "D Minor 9th"),  # D-F-A-C-E
        ([55, 59, 62, 65, 69], "G Dominant 9th"),  # G-B-D-F-A
        ([57, 60, 64, 67, 71], "A Minor 11th"),  # A-C-E-G-B
        ([64, 67, 71, 74, 77], "E Dominant 11th"),  # E-G#-B-D#-A

        # Exotic Chords
        ([60, 66, 72], "C Diminished 5th"),  # C-Eb-Gb
        ([60, 68, 75], "C Augmented 7th"),  # C-E-G#-Bb
        ([60, 64, 70], "C6"),  # C-E-G-A
        ([60, 63, 67], "C Minor"),  # C-Eb-G
        ([60, 63, 66, 69], "C Half-Diminished 7th"),  # C-Eb-Gb-Bb

        # Mystic Chord (exotic)
        ([60, 66, 72, 78, 84], "C Mystic Chord")  # C-F#-G-C-F
    ]

    music_controller = RTPMusicController()
    chord_finder = ChordFinder(music_controller)

    # Test all chords
    for midi_notes, expected_name in test_chords:
        result = chord_finder.identify_chord(midi_notes)
        print(f"Input: {midi_notes}, Expected: {expected_name}, Result: {result}")

