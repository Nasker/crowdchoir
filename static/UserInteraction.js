export default class GyroController {
    constructor(synth, onValuesChanged, onStatus) {
        this.synth = synth;
        this.onValuesChanged = onValuesChanged;
        this.onStatus = onStatus;
        this.isActive = false;
        this.isAvailable = this._detectSupport();
        this._orientationHandler = this._handleOrientation.bind(this);
        this._betaRange = 90;   // front/back tilt is clamped to +/- 90 degrees
        this._gammaRange = 45;  // left/right tilt is clamped to +/- 45 degrees
        this._eventReceived = false;
        this.yaw = 0;           // normalized yaw (0..1) for chord voicing
    }

    getYaw() {
        return this.yaw;
    }

    _detectSupport() {
        return typeof window !== 'undefined' &&
               ('DeviceOrientationEvent' in window || 'ondeviceorientation' in window);
    }

    _setStatus(message) {
        console.log('Gyro status:', message);
        if (this.onStatus) this.onStatus(message);
    }

    async enable() {
        if (!this.isAvailable) {
            this._setStatus('Gyroscope not available on this device/browser.');
            return false;
        }

        if (typeof DeviceOrientationEvent.requestPermission === 'function') {
            try {
                const state = await DeviceOrientationEvent.requestPermission();
                if (state !== 'granted') {
                    this._setStatus('Gyroscope permission denied.');
                    return false;
                }
            } catch (err) {
                this._setStatus('Gyroscope permission error: ' + err.message);
                return false;
            }
        }

        this._eventReceived = false;
        window.addEventListener('deviceorientation', this._orientationHandler);
        this.isActive = true;
        this._setStatus('Gyroscope enabled. Tilt device to control filter.');

        // Confirm events are actually firing; on insecure origins they may be silent.
        setTimeout(() => {
            if (this.isActive && !this._eventReceived) {
                this._setStatus('No gyroscope events received. On mobile, this usually requires HTTPS.');
            }
        }, 1500);

        return true;
    }

    disable() {
        window.removeEventListener('deviceorientation', this._orientationHandler);
        this.isActive = false;
        this._setStatus('Gyroscope disabled. XY pad active.');
    }

    async toggle() {
        if (this.isActive) {
            this.disable();
            return false;
        }
        return await this.enable();
    }

    _handleOrientation(event) {
        this._eventReceived = true;

        if (event.alpha != null) {
            const alpha = ((event.alpha % 360) + 360) % 360;
            this.yaw = alpha / 360;
        }

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
            this.onValuesChanged(cutoff, resonance, this.yaw);
        }
    }
}
