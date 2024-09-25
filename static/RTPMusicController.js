import RTPChordMatrix from './RTPChordMatrix.js';
import RTPScaleMatrix from './RTPScaleMatrix.js';

const PartType = {
    SCALE: 0,
    FULL_CHORD: 1,
    ARP_CHORD: 2,
    DRUM: 3
};

const scale_name = ["Chromatic", "Ionian", "Dorian", "Phrygian", "Lydian",
    "Mixolydian", "Aeolian", "Locrian", "Harmonic", "Gipsy",
    "Hawaian", "Blues", "Japanese", "Drum"];

const chord_name = ["mono", "octave", "powerchord", "Major", "minor", "Major7th",
    "minor7th", "Dominant7th", "Diminished", "Augmented", "Hendrixian",
    "Suspended2th", "Suspended4th", "Dominant9th", "Dominant11th", "Mystic"];

const root_name = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

class RTPMusicController {
    constructor() {
        this._CNote = 36;
        this._currentNote = this._CNote;
        this._lastNote = this._currentNote;
        this._currentRootNote = this._CNote;
        this._currentStep = 0;
        this._currentChordStep = 0;
        this._currentOctave = 0;
        this._octaveOffset = 0;
        this._currentScale = 0;
        this._currentChord = 0;
        this._lastChord = this._currentChord;
        this._voices = 1;
        this._numberOctaves = 2;
        this._velocity = 100;
        this._midiChannel = 0x90;

        this.scales = new RTPScaleMatrix();
        this.chords = new RTPChordMatrix();
    }

    set_current_scale(current_scale) {
        this._currentScale = current_scale;
        this.scales.set_tonality(this._currentScale);
    }

    set_current_root(current_root) {
        this._currentRootNote = current_root;
    }

    set_current_note(current_note) {
        this._currentNote = current_note;
    }

    set_current_octave(current_octave) {
        this._currentOctave = current_octave;
    }

    set_current_chord(current_chord) {
        this._currentChord = current_chord;
        this.chords.setArpChordType(current_chord);
    }

    up_octave() {
        if (this._octaveOffset < 5) {
            this._octaveOffset += 1;
        }
    }

    down_octave() {
        if (this._octaveOffset > 0) {
            this._octaveOffset -= 1;
        }
    }

    get_current_midi_note() {
        return (this._currentRootNote +
                this.scales.get_scale_step(this._currentStep) +
                this._currentOctave * 12 +
                this._octaveOffset * 12);
    }

    set_current_chord_step(step) {
        this._currentChordStep = step;
    }

    get_current_chord_midi_note() {
        return (this._currentRootNote +
                this.chords.getChordStep(this._currentChordStep) +
                this._currentOctave * 12 +
                this._octaveOffset * 12);
    }

    get_current_arp_chord_midi_note() {
        return (this._currentRootNote +
                this.chords.getArpChordStep(this._currentChordStep) +
                this._currentOctave * 12 +
                this._octaveOffset * 12);
    }

    get_current_root_note_name() {
        let root_index = this._currentRootNote % 12;
        return root_name[root_index];
    }

    get_scale_name() {
        return scale_name[this._currentScale];
    }

    get_chord_name() {
        return chord_name[this._currentChord];
    }
}

export default RTPMusicController;