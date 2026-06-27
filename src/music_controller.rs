use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Debug, Clone, Deserialize)]
pub struct ChordsData {
    pub names: Vec<String>,
    #[serde(rename = "chordStep")]
    pub chord_step: Vec<Vec<i32>>,
    #[serde(rename = "arpChordStep")]
    pub arp_chord_step: Vec<Vec<i32>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScalesData {
    pub names: Vec<String>,
    #[serde(rename = "rootNames")]
    pub root_names: Vec<String>,
    #[serde(rename = "toneStep")]
    pub tone_step: Vec<Vec<i32>>,
}

pub static CHORDS_DATA: LazyLock<ChordsData> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../static/data/chords.json")).unwrap()
});

pub static SCALES_DATA: LazyLock<ScalesData> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../static/data/scales.json")).unwrap()
});

#[derive(Debug, Clone, Default)]
pub struct ChordMatrix {
    chord_type: usize,
    n_steps: usize,
    n_arp_steps: usize,
    number_chords: usize,
}

impl ChordMatrix {
    pub fn new() -> Self {
        let data = &*CHORDS_DATA;
        let chord_type = 0;
        Self {
            chord_type,
            n_steps: data.chord_step[chord_type].len(),
            n_arp_steps: data.arp_chord_step[chord_type].len(),
            number_chords: data.chord_step[chord_type].len(),
        }
    }

    pub fn set_chord_type(&mut self, chord_type: usize) {
        let data = &*CHORDS_DATA;
        self.chord_type = chord_type;
        self.n_steps = data.chord_step[chord_type].len();
    }

    pub fn set_arp_chord_type(&mut self, chord_type: usize) {
        let data = &*CHORDS_DATA;
        self.chord_type = chord_type;
        self.n_arp_steps = data.arp_chord_step[chord_type].len();
    }

    pub fn get_chord_type(&self) -> usize {
        self.chord_type
    }

    pub fn get_chord_step(&self, step: usize) -> i32 {
        CHORDS_DATA.chord_step[self.chord_type][step]
    }

    pub fn get_arp_chord_step(&self, step: usize) -> i32 {
        CHORDS_DATA.arp_chord_step[self.chord_type][step]
    }

    pub fn get_chord_steps(&self) -> usize {
        self.n_steps
    }

    pub fn get_arp_chord_steps(&self) -> usize {
        self.n_arp_steps
    }

    pub fn get_number_chords(&self) -> usize {
        self.number_chords
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScalesMatrix {
    tonality: usize,
    number_scales: usize,
}

impl ScalesMatrix {
    pub fn new() -> Self {
        let data = &*SCALES_DATA;
        Self {
            tonality: 0,
            number_scales: data.tone_step.len(),
        }
    }

    pub fn set_tonality(&mut self, tonality: usize) {
        self.tonality = tonality;
    }

    pub fn get_tonality(&self) -> usize {
        self.tonality
    }

    pub fn get_steps(&self) -> usize {
        let data = &*SCALES_DATA;
        data.tone_step[self.tonality].len()
    }

    pub fn get_scale_step(&self, step_scale: usize) -> i32 {
        let data = &*SCALES_DATA;
        data.tone_step[self.tonality][step_scale]
    }

    pub fn get_number_scales(&self) -> usize {
        self.number_scales
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PartType {
    #[default]
    Scale = 0,
    FullChord = 1,
    ArpChord = 2,
    Drum = 3,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct MusicController {
    c_note: i32,
    current_note: i32,
    last_note: i32,
    current_root_note: i32,
    current_step: i32,
    current_chord_step: i32,
    current_octave: i32,
    octave_offset: i32,
    current_scale: usize,
    current_chord: usize,
    last_chord: usize,
    voices: i32,
    number_octaves: i32,
    velocity: i32,
    midi_channel: i32,
    pub scales: ScalesMatrix,
    pub chords: ChordMatrix,
}

impl MusicController {
    pub fn new() -> Self {
        let c_note = 36;
        Self {
            c_note,
            current_note: c_note,
            last_note: c_note,
            current_root_note: c_note,
            current_step: 0,
            current_chord_step: 0,
            current_octave: 0,
            octave_offset: 0,
            current_scale: 0,
            current_chord: 0,
            last_chord: 0,
            voices: 1,
            number_octaves: 2,
            velocity: 100,
            midi_channel: 0x90,
            scales: ScalesMatrix::new(),
            chords: ChordMatrix::new(),
        }
    }

    pub fn set_current_scale(&mut self, current_scale: usize) {
        self.current_scale = current_scale;
        self.scales.set_tonality(current_scale);
    }

    pub fn set_current_root(&mut self, current_root: i32) {
        self.current_root_note = current_root;
    }

    pub fn set_current_note(&mut self, current_note: i32) {
        self.current_note = current_note;
    }

    pub fn set_current_chord(&mut self, current_chord: usize) {
        self.current_chord = current_chord;
        self.chords.set_arp_chord_type(current_chord);
    }

    pub fn up_octave(&mut self) {
        if self.octave_offset < 5 {
            self.octave_offset += 1;
        }
    }

    pub fn down_octave(&mut self) {
        if self.octave_offset > 0 {
            self.octave_offset -= 1;
        }
    }

    pub fn get_current_midi_note(&self) -> i32 {
        self.current_root_note
            + self.scales.get_scale_step(self.current_step as usize)
            + self.current_octave * 12
            + self.octave_offset * 12
    }

    pub fn get_current_chord_midi_note(&self) -> i32 {
        self.current_root_note
            + self.chords.get_chord_step(self.current_chord_step as usize)
            + self.current_octave * 12
            + self.octave_offset * 12
    }

    pub fn get_current_arp_chord_midi_note(&self) -> i32 {
        self.current_root_note
            + self.chords.get_arp_chord_step(self.current_chord_step as usize)
            + self.current_octave * 12
            + self.octave_offset * 12
    }

    pub fn get_current_root_note_name(&self) -> &str {
        let root_index = (self.current_root_note % 12) as usize;
        &SCALES_DATA.root_names[root_index]
    }

    pub fn get_scale_name(&self) -> &str {
        &SCALES_DATA.names[self.current_scale]
    }

    pub fn get_chord_name(&self) -> &str {
        &CHORDS_DATA.names[self.current_chord]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chord_data_loads() {
        assert_eq!(CHORDS_DATA.names.len(), 16);
        assert_eq!(CHORDS_DATA.chord_step.len(), 16);
        assert_eq!(CHORDS_DATA.arp_chord_step.len(), 16);
    }

    #[test]
    fn scales_data_loads() {
        assert_eq!(SCALES_DATA.names.len(), 14);
        assert_eq!(SCALES_DATA.root_names.len(), 12);
        assert_eq!(SCALES_DATA.tone_step.len(), 14);
    }

    #[test]
    fn chord_matrix_initial_state() {
        let cm = ChordMatrix::new();
        assert_eq!(cm.get_chord_type(), 0);
        assert_eq!(cm.get_chord_steps(), 4); // mirrors Python bug
        assert_eq!(cm.get_arp_chord_steps(), 1);
    }

    #[test]
    fn scales_matrix_steps() {
        let mut sm = ScalesMatrix::new();
        sm.set_tonality(1); // Ionian
        assert_eq!(sm.get_steps(), 8);
        assert_eq!(sm.get_scale_step(0), 0);
        assert_eq!(sm.get_scale_step(1), 2);
    }

    #[test]
    fn music_controller_names() {
        let mut mc = MusicController::new();
        mc.set_current_chord(1);
        assert_eq!(mc.get_chord_name(), "Major");
        mc.set_current_scale(1);
        assert_eq!(mc.get_scale_name(), "Ionian");
        assert_eq!(mc.get_current_root_note_name(), "C");
    }
}
