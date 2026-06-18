const _chords = window.MUSIC_DATA.chords;

class RTPChordMatrix {
    constructor() {
        this.chordStep    = _chords.chordStep;
        this.arpChordStep = _chords.arpChordStep;
        this._chordType   = 0;
        this._stepChord   = 0;
        this._numberChords = this.chordStep.length;
        this._nSteps      = this.chordStep[this._chordType].length;
        this._nArpSteps   = this.arpChordStep[this._chordType].length;
    }

    setChordType(chordType) {
        this._chordType = chordType;
        this._nSteps = this.chordStep[this._chordType].length;
    }

    setArpChordType(chordType) {
        this._chordType = chordType;
        this._nArpSteps = this.arpChordStep[this._chordType].length;
    }

    getChordType()          { return this._chordType; }
    getChordStep(step)      { return this.chordStep[this._chordType][step]; }
    getArpChordStep(step)   { return this.arpChordStep[this._chordType][step]; }
    getChordSteps()         { return this._nSteps; }
    getArpChordSteps()      { return this._nArpSteps; }
    getNumberChords()       { return this._numberChords; }
}

export default RTPChordMatrix;
