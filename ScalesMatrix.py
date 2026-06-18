import json
import os

_data = json.load(open(os.path.join(os.path.dirname(__file__), 'static', 'data', 'scales.json')))

SCALE_NAMES = _data['names']
ROOT_NAMES  = _data['rootNames']
TONE_STEP   = _data['toneStep']


class RTPDiatonicMatrix:
    def __init__(self):
        self.toneStep      = TONE_STEP
        self._tonality     = 0
        self._nSteps       = 0
        self._stepScale    = 0
        self._step         = 0
        self._numberScales = len(self.toneStep)

    def set_tonality(self, tonality):
        self._tonality = tonality

    def get_tonality(self):
        return self._tonality

    def get_steps(self):
        return len(self.toneStep[self._tonality])

    def get_scale_step(self, step_scale):
        self._stepScale = step_scale
        self._step = self.toneStep[self._tonality][self._stepScale]
        return self._step

    def get_number_scales(self):
        return self._numberScales
