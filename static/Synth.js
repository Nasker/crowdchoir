export default class Synth {
    constructor() {
        this.synth = new Tone.Synth();
        this.feedbackDelay = new Tone.FeedbackDelay("8n", 0.5).toDestination();
        this.filter = new Tone.Filter({
            type: 'lowpass',
            frequency: 350,
            Q: 1
        });

        this.synth.connect(this.filter);
        this.filter.connect(this.feedbackDelay);
        this.bluesScale = ["C4", "Eb4", "F4", "F#4", "G4", "Bb4", "C5"];
        this.minFreq = Math.log(20);
        this.maxFreq = Math.log(20000);
    }

    playNoteFromPosition(x, y) {
        const noteIndex = Math.floor((x / window.innerWidth) * this.bluesScale.length);
        const note = this.bluesScale[noteIndex];
        const scaleY = 1 - (y / window.innerHeight);
        const frequency = Math.exp(scaleY * (this.maxFreq - this.minFreq) + this.minFreq);
        this.filter.frequency.value = frequency;
        this.synth.triggerRelease();
        this.synth.triggerAttackRelease(note, "64n");
    }
}