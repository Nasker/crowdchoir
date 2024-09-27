class RTPChordMatrix:
    """
    A class to manage chords and arpeggios.
    """

    # Class variables for chord and arpeggio steps
    chordStep = [
        [0, 12, -12, 24],  # Monophonic
        [0, 4, 7, 12, -12, 16, 19],  # Major
        [0, 3, 7, 12, -12, 15, 19],  # Minor
        [0, 4, 7, 11, -12, 16, 19],  # Major 7th
        [0, 3, 7, 10, -12, 15, 19],  # Minor 7th
        [0, 4, 7, 10, -12, 16, 19],  # Dominant 7th
        [0, 3, 6, 12, -12, 15, 18],  # Diminished
        [0, 3, 6, 9, 12, -12, 15, 18],  # Diminished 7th
        [0, 3, 6, 10, 12, -12, 15, 18],  # Half Diminished 7th
        [0, 4, 8, 12, -12, 16, 20],  # Augmented
        [0, 4, 7, 10, 14, 12, -12],  # Major 9th
        [0, 3, 7, 10, 14, 12, -12],  # Minor 9th
        [0, 4, 7, 10, 14, 12, -12],  # Dominant 9th
        [0, 5, 7, 12, -12, 24, 17, 19],  # Suspended 4th
        [0, 2, 7, 12, -12, 24, 17, 19],  # Suspended 2th
        [0, 4, 7, 9, 12, -12, 16, 19]  # Sixth
    ]

    arpChordStep = [
        [0],                # Monophonic
        [0, 4, 7],          # Major
        [0, 3, 7],          # Minor
        [0, 4, 7, 11],      # Major 7th
        [0, 3, 7, 10],      # Minor 7th
        [0, 4, 7, 10],      # Dominant 7th
        [0, 3, 6],          # Diminished
        [0, 3, 6, 9],       # Diminished 7th
        [0, 3, 6, 10],      # Half-Diminished 7th
        [0, 4, 8],          # Augmented
        [0, 4, 7, 14],      # Major 9th
        [0, 3, 7, 14],      # Minor 9th
        [0, 4, 7, 10, 14],  # Dominant 9th
        [0, 5, 7],          # Suspended 4th
        [0, 2, 7],          # Suspended 2nd
        [0, 4, 7, 9],       # 6th chord
    ]

    def __init__(self):
        self._chordType = 0
        self._stepChord = 0
        self._numberChords = len(self.chordStep)
        self._nSteps = len(self.chordStep[self._chordType])  # Automatically set
        self._nArpSteps = len(self.arpChordStep[self._chordType])  # Automatically set

    def setChordType(self, chordType):
        """
        Set the chord type and dynamically update the number of steps.
        """
        self._chordType = chordType
        self._nSteps = len(self.chordStep[self._chordType])

    def setArpChordType(self, chordType):
        """
        Set the arpeggio chord type and dynamically update the number of arpeggio steps.
        """
        self._chordType = chordType
        self._nArpSteps = len(self.arpChordStep[self._chordType])

    def getChordType(self):
        """
        Returns the current chord type.
        """
        return self._chordType

    def getChordStep(self, step):
        """
        Returns the specific chord step for the current chord type.
        """
        return self.chordStep[self._chordType][step]

    def getArpChordStep(self, step):
        """
        Returns the specific arpeggio step for the current arpeggio type.
        """
        return self.arpChordStep[self._chordType][step]

    def getChordSteps(self):
        """
        Returns the number of chord steps for the current chord type.
        """
        return self._nSteps

    def getArpChordSteps(self):
        """
        Returns the number of arpeggio steps for the current chord type.
        """
        return self._nArpSteps

    def getNumberChords(self):
        """
        Returns the total number of available chord types.
        """
        return self._numberChords


# Example usage
if __name__ == "__main__":
    chord_matrix = RTPChordMatrix()
    chord_matrix.setChordType(3)  # Setting to Major
    print(f"Chord Type: {chord_matrix.getChordType()}")
    print(f"Number of Chord Steps: {chord_matrix.getChordSteps()}")
    print(f"Chord Step 0: {chord_matrix.getChordStep(0)}")
    chord_matrix.setArpChordType(3)  # Setting to Major
    print(f"Number of Arpeggio Steps: {chord_matrix.getArpChordSteps()}")
    print(f"Arpeggio Step 0: {chord_matrix.getArpChordStep(0)}")
