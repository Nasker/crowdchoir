const _scales = window.MUSIC_DATA.scales;

class RTPScaleMatrix {
    constructor() {
        this.toneStep      = _scales.toneStep;
        this._tonality     = 0;
        this._nSteps       = 0;
        this._stepScale    = 0;
        this._step         = 0;
        this._numberScales = this.toneStep.length;
    }

    set_tonality(tonality)      { this._tonality = tonality; }
    get_tonality()              { return this._tonality; }
    get_steps()                 { return this.toneStep[this._tonality].length; }
    get_scale_step(step_scale)  { this._stepScale = step_scale; this._step = this.toneStep[this._tonality][this._stepScale]; return this._step; }
    get_number_scales()         { return this._numberScales; }
}

export default RTPScaleMatrix;
