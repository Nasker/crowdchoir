export default class Synth {
    constructor(musicController) {
        this.musicController = musicController;
        this.synth = new Tone.Sampler({
            urls: {
                C3: "C3.mp3",
                E3: "E3.mp3",
                G3: "G3.mp3",
                A3: "A3.mp3",
                C4: "A4.mp3",
            },
            baseUrl: "/static/samples/",
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

    setFilterFrequency(y) {
        const frequency = Math.exp((y / window.innerHeight) * (this.maxFreq - this.minFreq) + this.minFreq);
        this.filter.frequency.value = frequency;
    }

    playNoteOnFromPosition(x) {
        const noteIndex = Math.floor((x / window.innerWidth) * this.musicController.chords.getChordSteps());
        this.musicController.set_current_chord_step(noteIndex);
        const noteFreq = Tone.Frequency(this.musicController.get_current_chord_midi_note(), "midi").toFrequency();
        this.synth.triggerAttack(noteFreq);
        console.log("Playing note:", noteFreq, 0.0, 0.1);
    }

    playNoteOff() {
        console.log("Stopping note");
        this.synth.triggerRelease();
    }
}