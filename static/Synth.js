export default class Synth {
    constructor(musicController) {
        this.musicController = musicController;
        this.currentSampleSet = 'mello_flute';
        
        // Define available sample sets
        this.sampleSets = {
            'mello_flute': "/static/samples/mello_flute/",
            'mello_ohs': "/static/samples/mello_ohs/"
        };
        
        // Create the sampler with default sample set
        this.synth = new Tone.Sampler({
            urls: {
                C3: "C3.mp3",
                E3: "E3.mp3",
                G3: "G3.mp3",
                A3: "A3.mp3",
                C4: "C4.mp3",
            },
            baseUrl: this.sampleSets[this.currentSampleSet],
        });
        this.limiter = new Tone.Limiter(-20).toDestination();
        this.comp = new Tone.Compressor(-30, 3);
        this.feedbackDelay = new Tone.FeedbackDelay("8n", 0.1);
        this.envelope = new Tone.AmplitudeEnvelope({
            attack: 1.0,
            decay: 0.2,
            sustain: 1.0,
            release: 0.8,
        });
        this.filter = new Tone.Filter({
            type: 'lowpass',
            frequency: 1000,
            Q: 1
        });
        this.synth.connect(this.filter);
        this.filter.connect(this.envelope);
        this.envelope.connect(this.comp);
        this.filter.connect(this.comp);
        this.comp.connect(this.feedbackDelay);
        this.feedbackDelay.connect(this.limiter);
        this.minFreq = Math.log(20);
        this.maxFreq = Math.log(20000);
        this.lastNote = null;
    }

    /**
     * Set the filter cutoff frequency based on Y position
     * @param {number} normalizedY - Value between 0 and 1 representing vertical position
     * @returns {number} - The actual frequency value set
     */
    setFilterFrequency(normalizedY) {
        // Invert Y so higher position = higher frequency (1 at top, 0 at bottom)
        normalizedY = 1 - normalizedY;
        
        // Map to frequency range (exponential scale sounds more natural)
        const minFreq = 80;    // 80 Hz
        const maxFreq = 12000; // 12 kHz
        
        // Use exponential mapping for more natural frequency control
        const expScale = Math.pow(maxFreq / minFreq, normalizedY);
        const frequency = minFreq * expScale;
        
        // Apply frequency with rounding for display purposes
        this.filter.frequency.value = frequency;
        
        // Return rounded value for display
        return Math.round(frequency);
    }
    
    /**
     * Set the filter resonance (Q) based on X position
     * @param {number} normalizedX - Value between 0 and 1 representing horizontal position
     * @returns {number} - The actual Q value set
     */
    setResonance(normalizedX) {
        // Map X to Q value range (0.1 to 10)
        const minQ = 0.1;
        const maxQ = 10;
        
        // Use a curve that gives finer control at lower values
        const qValue = minQ + (Math.pow(normalizedX, 2) * (maxQ - minQ));
        
        // Apply Q value
        this.filter.Q.value = qValue;
        
        // Return value with 1 decimal place for display
        return Math.round(qValue * 10) / 10;
    }

    playNoteOnFromPosition(x) {
        const noteIndex = Math.floor((x / window.innerWidth) * this.musicController.chords.getChordSteps());
        this.musicController.set_current_chord_step(noteIndex);
        const noteFreq = Tone.Frequency(this.musicController.get_current_chord_midi_note(), "midi").toFrequency();
        if(this.lastNote){
            this.synth.triggerRelease(this.lastNote);
            this.envelope.triggerRelease();
        }
        this.synth.triggerAttack(noteFreq);
        this.envelope.triggerAttack();
        this.lastNote = noteFreq;
        console.log("Playing note:", noteFreq);
    }

    playNoteOff() {
        console.log("Stopping note");
        this.synth.triggerRelease(this.lastNote);
        this.envelope.triggerRelease();
    }
    
    /**
     * Changes the sample set used by the synth
     * @param {string} sampleSetName - Name of the sample set to use ('mello_flute', 'piano', or 'strings')
     * @returns {boolean} - Whether the change was successful
     */
    changeSampleSet(sampleSetName) {
        // Check if the requested sample set exists
        if (!this.sampleSets[sampleSetName]) {
            console.error(`Sample set '${sampleSetName}' not found`);
            return false;
        }
        
        // If we're already using this sample set, do nothing
        if (this.currentSampleSet === sampleSetName) {
            return true;
        }
        
        // Stop any currently playing notes
        if (this.lastNote) {
            this.playNoteOff();
        }
        
        // Update the current sample set
        this.currentSampleSet = sampleSetName;
        
        // Disconnect the old sampler
        this.synth.disconnect();
        
        // Create a new sampler with the selected sample set
        this.synth = new Tone.Sampler({
            urls: {
                C3: "C3.mp3",
                E3: "E3.mp3",
                G3: "G3.mp3",
                A3: "A3.mp3",
                C4: "C4.mp3",
            },
            baseUrl: this.sampleSets[sampleSetName],
            onload: () => {
                console.log(`Loaded sample set: ${sampleSetName}`);
            }
        });
        
        // Reconnect the signal chain
        this.synth.connect(this.filter);
        
        console.log(`Changed sample set to: ${sampleSetName}`);
        return true;
    }
}