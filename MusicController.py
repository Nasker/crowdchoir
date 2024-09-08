from ChordMatrix import RTPChordMatrix
from ScalesMatrix import RTPDiatonicMatrix
from enum import Enum


class PartType(Enum):
    SCALE = 0
    FULL_CHORD = 1
    ARP_CHORD = 2
    DRUM = 3

# Define the scale names (list of strings in Python)
scale_name = ["Chromatic", "Ionian", "Dorian", "Phrygian", "Lydian",
    "Mixolydian", "Aeolian", "Locrian", "Harmonic", "Gipsy",
    "Hawaian", "Blues", "Japanese", "Drum"]

# Define the chord names (list of strings in Python)
chord_name = ["mono", "octave", "powerchord", "Major", "minor", "Major7th",
    "minor7th", "Dominant7th", "Diminished", "Augmented", "Hendrixian",
    "Suspended2th", "Suspended4th", "Dominant9th", "Dominant11th", "Mystic"]

# Define the root names (list of strings in Python)
root_name = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]
class RTPMusicController:
    def __init__(self):
        self._CNote = 36
        self._currentNote = self._CNote
        self._lastNote = self._currentNote
        self._currentRootNote = self._CNote
        self._currentStep = 0
        self._currentChordStep = 0
        self._currentOctave = 0
        self._octaveOffset = 0
        self._currentScale = 0
        self._currentChord = 0
        self._lastChord = self._currentChord
        self._voices = 1
        self._numberOctaves = 2
        self._velocity = 100
        self._midiChannel = 0x90

        self.scales = RTPDiatonicMatrix()
        self.chords = RTPChordMatrix()

    # Simplified setters where redundant setters are removed
    def set_current_scale(self, current_scale):
        self._currentScale = current_scale
        self.scales.set_tonality(self._currentScale)

    def set_current_chord(self, current_chord):
        self._currentChord = current_chord
        self.chords.setArpChordType(current_chord)

    def up_octave(self):
        if self._octaveOffset < 5:
            self._octaveOffset += 1

    def down_octave(self):
        if self._octaveOffset > 0:
            self._octaveOffset -= 1

    # Simplified getters
    def get_current_midi_note(self):
        return (self._currentRootNote +
                self.scales.get_scale_step(self._currentStep) +
                self._currentOctave * 12 +
                self._octaveOffset * 12)

    def get_current_chord_midi_note(self):
        return (self._currentRootNote +
                self.chords.getChordStep(self._currentChordStep) +
                self._currentOctave * 12 +
                self._octaveOffset * 12)

    def get_current_arp_chord_midi_note(self):
        return (self._currentRootNote +
                self.chords.getArpChordStep(self._currentChordStep) +
                self._currentOctave * 12 +
                self._octaveOffset * 12)

    def get_current_root_note_name(self):
        root_index = self._currentRootNote - self._CNote
        return root_name[root_index]

    def get_scale_name(self):
        return scale_name[self._currentScale]

    def get_chord_name(self):
        return chord_name[self._currentChord]

