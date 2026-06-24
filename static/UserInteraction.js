export default class GyroController {
    constructor(synth, onValuesChanged) {
        this.synth = synth;
        this.onValuesChanged = onValuesChanged;
        this.isActive = false;
        this.isAvailable = this._detectSupport();
        this._orientationHandler = this._handleOrientation.bind(this);
        this._betaRange = 90;   // front/back tilt is clamped to +/- 90 degrees
        this._gammaRange = 45;  // left/right tilt is clamped to +/- 45 degrees
    }

    _detectSupport() {
        return typeof window !== 'undefined' &&
               ('DeviceOrientationEvent' in window || 'ondeviceorientation' in window);
    }

    async enable() {
        if (!this.isAvailable) return false;

        if (typeof DeviceOrientationEvent.requestPermission === 'function') {
            try {
                const state = await DeviceOrientationEvent.requestPermission();
                if (state !== 'granted') {
                    console.warn('Gyroscope permission denied:', state);
                    return false;
                }
            } catch (err) {
                console.error('Gyroscope permission error:', err);
                return false;
            }
        }

        window.addEventListener('deviceorientation', this._orientationHandler);
        this.isActive = true;
        return true;
    }

    disable() {
        window.removeEventListener('deviceorientation', this._orientationHandler);
        this.isActive = false;
    }

    async toggle() {
        if (this.isActive) {
            this.disable();
            return false;
        }
        return await this.enable();
    }

    _handleOrientation(event) {
        if (event.beta == null || event.gamma == null) return;

        // beta -> filter cutoff (0..1), inverted so top of phone = high cutoff
        const beta = Math.max(-this._betaRange, Math.min(this._betaRange, event.beta));
        const normalizedY = 1 - ((beta + this._betaRange) / (this._betaRange * 2));

        // gamma -> resonance (0..1)
        const gamma = Math.max(-this._gammaRange, Math.min(this._gammaRange, event.gamma));
        const normalizedX = (gamma + this._gammaRange) / (this._gammaRange * 2);

        const cutoff = this.synth.setFilterFrequency(normalizedY);
        const resonance = this.synth.setResonance(normalizedX);

        if (this.onValuesChanged) {
            this.onValuesChanged(cutoff, resonance);
        }
    }
}
