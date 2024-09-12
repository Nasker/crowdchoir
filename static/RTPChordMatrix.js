class RTPChordMatrix {
    constructor() {
        this.chordStep = [
            [0, 12, -12, 24],       // Monophonic
            [0, 12, -12, 24],       // Octave
            [0, 7, 12, -12, 19],    // PowerChord
            [0, 4, 7, 12, -12, 16, 19],  // Major
            [0, 3, 7, 12, -12, 15, 19],  // Minor
            [0, 4, 7, 11, -12, 16, 19],  // Major 7th
            [0, 3, 7, 10, -12, 15, 19],  // Minor 7th
            [0, 4, 7, 10, -12, 16, 19],  // Dominant 7th
            [0, 3, 6, 12, -12, 15, 18],  // Diminished
            [0, 4, 8, 12, -12, 16, 20],  // Augmented
            [0, 4, 10, 15, -12, 16, 22],  // Hendrixian
            [0, 3, 6, 12, -12, 15, 18],  // Sus2
            [0, 5, 7, 12, -12, 24, 17, 19],  // Sus4
            [0, 4, 8, 12, -12, 24, 16, 20],  // Dominant Ninth
            [0, 3, 8, 12, -12, 24, 15, 20],  // Dominant Ninth
            [0, 6, 10, 16, 21, 26, -12, 18]  // Mystic
        ];

        this.arpChordStep = [
            [0],                // Monophonic
            [0],                // Octave
            [0, 7],             // PowerChord
            [0, 4, 7],          // Major
            [0, 3, 7],          // Minor
            [0, 4, 7, 11],      // Major 7th
            [0, 3, 7, 10],      // Minor 7th
            [0, 4, 7, 10],      // Dominant 7th
            [0, 3, 6],          // Diminished
            [0, 4, 8],          // Augmented
            [0, 4, 10],         // Hendrixian
            [0, 3, 6],          // Sus2
            [0, 5, 7],          // Sus4
            [0, 4, 8],          // Dominant Ninth
            [0, 3, 8],          // Dominant Ninth
            [0, 6, 10]          // Mystic
        ];

        this._chordType = 0;
        this._stepChord = 0;
        this._numberChords = this.chordStep.length;
        this._nSteps = this.chordStep[this._chordType].length;
        this._nArpSteps = this.arpChordStep[this._chordType].length;
    }

    setChordType(chordType) {
        this._chordType = chordType;
        this._nSteps = this.chordStep[this._chordType].length;
    }

    setArpChordType(chordType) {
        this._chordType = chordType;
        this._nArpSteps = this.arpChordStep[this._chordType].length;
    }

    getChordType() {
        return this._chordType;
    }

    getChordStep(step) {
        return this.chordStep[this._chordType][step];
    }

    getArpChordStep(step) {
        return this.arpChordStep[this._chordType][step];
    }

    getChordSteps() {
        return this._nSteps;
    }

    getArpChordSteps() {
        return this._nArpSteps;
    }

    getNumberChords() {
        return this._numberChords;
    }
}

export default RTPChordMatrix;