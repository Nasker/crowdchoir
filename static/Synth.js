export default class Synth {
    constructor(musicController) {
        this.musicController = musicController;
        this.synth = new Tone.Sampler({
            urls: {
                C3: "C3.mp3",
                E3: "E3.mp3",
                G3: "G3.mp3",
                A3: "A3.mp3",
                C4: "C4.mp3",
            },
            baseUrl: "/static/samples/",
            envelope: {
                attack: 0.8,
                decay: 0.2,
                sustain: 0.5,
                release: 0.5
            },
            onload: () => {
                // Enable looping
                Object.keys(this.synth._buffers._buffers).forEach(key => {
                    this.synth._buffers._buffers[key]._buffer.loop = true;
                    this.synth._buffers._buffers[key]._buffer.loopStart = 0.1;
                    this.synth._buffers._buffers[key]._buffer.loopEnd = 0.5;
                });
            }
        });
        this.limiter = new Tone.Limiter(-10).toDestination();
        this.comp = new Tone.Compressor(-30, 3);
        this.feedbackDelay = new Tone.FeedbackDelay("8n", 0.1);
        this.filter = new Tone.Filter({
            type: 'lowpass',
            frequency: 350,
            Q: 1
        });
        this.synth.connect(this.filter);
        this.filter.connect(this.comp);
        this.comp.connect(this.feedbackDelay);
        this.feedbackDelay.connect(this.limiter);
        this.minFreq = Math.log(20);
        this.maxFreq = Math.log(20000);
        this.lastNote = null;
    }

    setFilterFrequency(y) {
        const frequency = Math.exp((y / window.innerHeight) * (this.maxFreq - this.minFreq) + this.minFreq);
        this.filter.frequency.value = frequency;
    }

    playNoteOnFromPosition(x) {
        const noteIndex = Math.floor((x / window.innerWidth) * this.musicController.chords.getChordSteps());
        this.musicController.set_current_chord_step(noteIndex);
        const noteFreq = Tone.Frequency(this.musicController.get_current_chord_midi_note(), "midi").toFrequency();
        if(this.lastNote)
            this.synth.triggerRelease(this.lastNote);
        this.synth.triggerAttack(noteFreq);
        this.lastNote = noteFreq;
        console.log("Playing note:", noteFreq);
    }

    playNoteOff() {
        console.log("Stopping note");
        this.synth.triggerRelease(this.lastNote);
    }
}