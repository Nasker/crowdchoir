export default class Synth {
    constructor(musicController) {
        this.musicController = musicController;
        this.synth = new Tone.Synth({
            envelope: {
                attack: 2.0,  // Time it takes to reach full volume
                decay: 0.2,   // Time it takes to drop to sustain level
                sustain: 1.0, // Sustain level (as a percentage of full volume)
                release: 2.0  // Time it takes to fade out after the note is released
            }
        });
        this.feedbackDelay = new Tone.FeedbackDelay("8n", 0.5).toDestination();
        this.filter = new Tone.Filter({
            type: 'lowpass',
            frequency: 350,
            Q: 1
        });

        this.synth.connect(this.filter);
        this.filter.connect(this.feedbackDelay);
        this.minFreq = Math.log(20);
        this.maxFreq = Math.log(20000);
    }

    playNoteFromPosition(x, y) {
        const noteIndex = Math.floor((x / window.innerWidth) * this.musicController.chords.getChordSteps());
        this.musicController.set_current_chord_step(noteIndex);
        const note = this.musicController.get_current_chord_midi_note();
        this.musicController.set
        console.log("Playing note:", note, "at index:", noteIndex);
        const scaleY = 1 - (y / window.innerHeight);
        const frequency = Math.exp(scaleY * (this.maxFreq - this.minFreq) + this.minFreq);
        this.filter.frequency.value = frequency;
        this.synth.triggerRelease();
        const noteFreq = Tone.Frequency(note, "midi").toFrequency();
        this.synth.triggerAttackRelease(noteFreq, "64n");
        console.log("Playing note:", noteFreq, "at frequency:", frequency);
    }
}