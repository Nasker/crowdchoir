class RTPDiatonicMatrix:
    # Define the toneStep matrix
    toneStep = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],  # Chromatic
        [0, 2, 4, 5, 7, 9, 11, 12],  # Ionian
        [0, 2, 3, 5, 7, 9, 10, 12],  # Dorian
        [0, 1, 3, 5, 7, 8, 10, 12],  # Phrygian
        [0, 2, 4, 6, 7, 9, 11, 12],  # Lydian
        [0, 2, 4, 5, 7, 9, 10, 12],  # Mixolydian
        [0, 2, 3, 5, 7, 8, 10, 12],  # Aeolian
        [0, 1, 3, 5, 6, 8, 10, 12],  # Locrian
        [0, 2, 3, 5, 7, 8, 11, 12],  # Harmonic Minor
        [0, 1, 4, 5, 7, 8, 10, 12],  # Spanish Gipsy
        [0, 2, 3, 5, 7, 9, 11, 12],  # Hawaiian
        [0, 3, 5, 6, 7, 10, 12],  # Blues
        [0, 1, 5, 7, 8, 12],  # Japanese
        [36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51]  # Drum
    ]
    def __init__(self):
        self._tonality = 0
        self._nSteps = 0
        self._stepScale = 0
        self._step = 0
        self._numberScales = len(self.toneStep)

    def set_tonality(self, tonality):
        """Set the current tonality (scale type)."""
        self._tonality = tonality

    def get_tonality(self):
        """Return the current tonality."""
        return self._tonality

    def get_steps(self):
        """Return the number of steps in the current tonality."""
        return len(self.toneStep[self._tonality])

    def get_scale_step(self, step_scale):
        """Return the corresponding scale step."""
        self._stepScale = step_scale
        self._step = self.toneStep[self._tonality][self._stepScale]
        return self._step

    def get_number_scales(self):
        """Return the total number of scales."""
        return self._numberScales
