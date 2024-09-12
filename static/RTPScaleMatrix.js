class RTPDiatonicMatrix {
    constructor() {
        this.toneStep = [
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],  // Chromatic
            [0, 2, 4, 5, 7, 9, 11, 12],  // Ionian
            [0, 2, 3, 5, 7, 9, 10, 12],  // Dorian
            [0, 1, 3, 5, 7, 8, 10, 12],  // Phrygian
            [0, 2, 4, 6, 7, 9, 11, 12],  // Lydian
            [0, 2, 4, 5, 7, 9, 10, 12],  // Mixolydian
            [0, 2, 3, 5, 7, 8, 10, 12],  // Aeolian
            [0, 1, 3, 5, 6, 8, 10, 12],  // Locrian
            [0, 2, 3, 5, 7, 8, 11, 12],  // Harmonic Minor
            [0, 1, 4, 5, 7, 8, 10, 12],  // Spanish Gipsy
            [0, 2, 3, 5, 7, 9, 11, 12],  // Hawaiian
            [0, 3, 5, 6, 7, 10, 12],  // Blues
            [0, 1, 5, 7, 8, 12],  // Japanese
            [36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51]  // Drum
        ];
        this._tonality = 0;
        this._nSteps = 0;
        this._stepScale = 0;
        this._step = 0;
        this._numberScales = this.toneStep.length;
    }

    set_tonality(tonality) {
        this._tonality = tonality;
    }

    get_tonality() {
        return this._tonality;
    }

    get_steps() {
        return this.toneStep[this._tonality].length;
    }

    get_scale_step(step_scale) {
        this._stepScale = step_scale;
        this._step = this.toneStep[this._tonality][this._stepScale];
        return this._step;
    }

    get_number_scales() {
        return this._numberScales;
    }
}

export default RTPDiatonicMatrix;