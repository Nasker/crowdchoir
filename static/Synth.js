export default class Synth {
    constructor() {
        // Create a single Tone.Synth object
        this.synth = new Tone.Synth();

        // Create a feedback delay effect
        this.feedbackDelay = new Tone.FeedbackDelay("8n", 0.5).toDestination();

        // Create a lowpass filter
        this.filter = new Tone.Filter({
            type: 'lowpass',
            frequency: 350,
            Q: 1
        });

        this.synth.connect(this.filter);
        this.filter.connect(this.feedbackDelay);
    }

    playNoteFromPosition(x, y) {
        const bluesScale = ["C4", "Eb4", "F4", "F#4", "G4", "Bb4", "C5"];
        const noteIndex = Math.floor((x / window.innerWidth) * bluesScale.length);
        const note = bluesScale[noteIndex];
        const minFreq = Math.log(20);
        const maxFreq = Math.log(20000);
        const scaleY = 1 - (y / window.innerHeight);
        const frequency = Math.exp(scaleY * (maxFreq - minFreq) + minFreq);
        this.filter.frequency.value = frequency;
        const time = Tone.now();
        this.synth.triggerAttackRelease(note, "8n", time);
    }
}