class RTPChordMatrix {
    constructor() {
        this.chordStep = [
            [0, 12, -12, 24],  // Monophonic
            [0, 4, 7, 12, -12, 16, 19],  // Major
            [0, 3, 7, 12, -12, 15, 19],  // Minor
            [0, 4, 7, 11, -12, 16, 19],  // Major 7th
            [0, 3, 7, 10, -12, 15, 19],  // Minor 7th
            [0, 4, 7, 10, -12, 16, 19],  // Dominant 7th
            [0, 3, 6, 12, -12, 15, 18],  // Diminished
            [0, 3, 6, 9, 12, -12, 15, 18],  // Diminished 7th
            [0, 3, 6, 10, 12, -12, 15, 18],  // Half Diminished 7th
            [0, 4, 8, 12, -12, 16, 20],  // Augmented
            [0, 4, 7, 10, 14, 12, -12],  // Major 9th
            [0, 3, 7, 10, 14, 12, -12],  // Minor 9th
            [0, 4, 7, 10, 14, 12, -12],  // Dominant 9th
            [0, 5, 7, 12, -12, 24, 17, 19],  // Suspended 4th
            [0, 2, 7, 12, -12, 24, 17, 19],  // Suspended 2th
            [0, 4, 7, 9, 12, -12, 16, 19]  // Sixth
        ];

        this.arpChordStep = [
            [0],                //Monophonic
            [0, 4, 7],          // Major
            [0, 3, 7],          // Minor
            [0, 4, 7, 11],      // Major 7th
            [0, 3, 7, 10],      // Minor 7th
            [0, 4, 7, 10],      // Dominant 7th
            [0, 3, 6],          // Diminished
            [0, 3, 6, 9],       // Diminished 7th
            [0, 3, 6, 10],      // Half-Diminished 7th
            [0, 4, 8],          // Augmented
            [0, 4, 7, 14],      // Major 9th
            [0, 3, 7, 14],      // Minor 9th
            [0, 4, 7, 10, 14],  // Dominant 9th
            [0, 5, 7],          // Suspended 4th
            [0, 2, 7],          // Suspended 2nd
            [0, 4, 7, 9],       // 6th chord
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