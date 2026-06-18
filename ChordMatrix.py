import json
import os

_data = json.load(open(os.path.join(os.path.dirname(__file__), 'static', 'data', 'chords.json')))

CHORD_NAMES    = _data['names']
CHORD_STEP     = _data['chordStep']
ARP_CHORD_STEP = _data['arpChordStep']


class RTPChordMatrix:
    def __init__(self):
        self.chordStep    = CHORD_STEP
        self.arpChordStep = ARP_CHORD_STEP
        self._chordType   = 0
        self._stepChord   = 0
        self._numberChords = len(self.chordStep)
        self._nSteps      = len(self.chordStep[self._chordType])
        self._nArpSteps   = len(self.arpChordStep[self._chordType])

    def setChordType(self, chordType):
        self._chordType = chordType
        self._nSteps = len(self.chordStep[self._chordType])

    def setArpChordType(self, chordType):
        self._chordType = chordType
        self._nArpSteps = len(self.arpChordStep[self._chordType])

    def getChordType(self):
        return self._chordType

    def getChordStep(self, step):
        return self.chordStep[self._chordType][step]

    def getArpChordStep(self, step):
        return self.arpChordStep[self._chordType][step]

    def getChordSteps(self):
        return self._nSteps

    def getArpChordSteps(self):
        return self._nArpSteps

    def getNumberChords(self):
        return self._numberChords
