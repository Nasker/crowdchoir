use crate::music_controller::{MusicController, CHORDS_DATA};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChordResult {
    pub root: u8,
    pub chord_type: u8,
}

pub struct ChordFinder {
    music_controller: MusicController,
}

impl ChordFinder {
    pub fn new() -> Self {
        Self {
            music_controller: MusicController::new(),
        }
    }

    pub fn midi_to_pitch_classes(&self, midi_notes: &[u8]) -> Vec<u8> {
        let mut pcs: Vec<u8> = midi_notes.iter().map(|n| n % 12).collect();
        pcs.sort_unstable();
        pcs.dedup();
        pcs
    }

    pub fn get_intervals(&self, notes: &[u8]) -> Vec<u8> {
        notes
            .windows(2)
            .map(|w| (w[1] as i16 - w[0] as i16).rem_euclid(12) as u8)
            .collect()
    }

    pub fn identify_chord(&mut self, midi_notes: &[u8]) -> ChordResult {
        let pitch_classes = self.midi_to_pitch_classes(midi_notes);
        if pitch_classes.is_empty() {
            return ChordResult::default();
        }

        for i in 0..pitch_classes.len() {
            let rotated: Vec<u8> = pitch_classes[i..]
                .iter()
                .chain(&pitch_classes[..i])
                .copied()
                .collect();
            let intervals = self.get_intervals(&rotated);
            for chord_type in 0..CHORDS_DATA.names.len() {
                self.music_controller.set_current_chord(chord_type);
                let n_steps = self.music_controller.chords.get_chord_steps();
                let steps: Vec<i32> = (0..n_steps)
                    .map(|s| self.music_controller.chords.get_chord_step(s))
                    .collect();
                let chord_intervals: Vec<u8> = steps
                    .windows(2)
                    .map(|w| (w[1] as i32 - w[0] as i32).rem_euclid(12) as u8)
                    .collect();

                let input_len = intervals.len();
                let chord_len = chord_intervals.len();
                if intervals[..chord_len.min(input_len)] == chord_intervals[..input_len.min(chord_len)]
                {
                    let root_note = rotated[0];
                    return ChordResult {
                        root: root_note,
                        chord_type: chord_type as u8,
                    };
                }
            }
        }

        ChordResult::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These expected values are the exact outputs produced by the original Python ChordFinder.
    #[test]
    fn python_self_test_suite() {
        let cases: Vec<(Vec<u8>, ChordResult)> = vec![
            // Basic triads and inversions
            (vec![60, 64, 67], ChordResult { root: 0, chord_type: 1 }),
            (vec![64, 67, 72], ChordResult { root: 0, chord_type: 1 }),
            (vec![67, 72, 76], ChordResult { root: 0, chord_type: 1 }),
            (vec![57, 60, 64], ChordResult { root: 9, chord_type: 2 }),
            (vec![60, 64, 69], ChordResult { root: 9, chord_type: 2 }),
            (vec![64, 69, 72], ChordResult { root: 9, chord_type: 2 }),
            // Seventh chords
            (vec![55, 59, 62, 65], ChordResult { root: 7, chord_type: 5 }),
            (vec![59, 62, 65, 67], ChordResult { root: 7, chord_type: 5 }),
            (vec![62, 65, 67, 71], ChordResult { root: 7, chord_type: 5 }),
            (vec![65, 67, 71, 74], ChordResult { root: 7, chord_type: 5 }),
            // Diminished
            (vec![62, 65, 68], ChordResult { root: 2, chord_type: 6 }),
            (vec![65, 68, 74], ChordResult { root: 2, chord_type: 6 }),
            (vec![68, 74, 77], ChordResult { root: 2, chord_type: 6 }),
            // Augmented (Python identifies as root C because of the interval symmetry)
            (vec![64, 68, 72], ChordResult { root: 0, chord_type: 9 }),
            (vec![68, 72, 76], ChordResult { root: 0, chord_type: 9 }),
            (vec![72, 76, 80], ChordResult { root: 0, chord_type: 9 }),
            // Major 7th
            (vec![65, 69, 72, 76], ChordResult { root: 5, chord_type: 3 }),
            (vec![69, 72, 76, 77], ChordResult { root: 5, chord_type: 3 }),
            (vec![72, 76, 77, 81], ChordResult { root: 5, chord_type: 3 }),
            (vec![76, 77, 81, 84], ChordResult { root: 5, chord_type: 3 }),
            // Suspended 4th
            (vec![60, 65, 67], ChordResult { root: 0, chord_type: 13 }),
            (vec![65, 67, 72], ChordResult { root: 0, chord_type: 13 }),
            // Diminished 7th (Python identifies as root D, not B)
            (vec![59, 62, 65, 68], ChordResult { root: 2, chord_type: 7 }),
            (vec![62, 65, 68, 71], ChordResult { root: 2, chord_type: 7 }),
            (vec![65, 68, 71, 74], ChordResult { root: 2, chord_type: 7 }),
            (vec![68, 71, 74, 77], ChordResult { root: 2, chord_type: 7 }),
            // Extended / exotic chords (Python returns (0,0) or mis-identifies these)
            (vec![60, 64, 67, 71, 74], ChordResult { root: 0, chord_type: 0 }),
            (vec![62, 65, 69, 72, 76], ChordResult { root: 5, chord_type: 15 }),
            (vec![55, 59, 62, 65, 69], ChordResult { root: 0, chord_type: 0 }),
            (vec![57, 60, 64, 67, 71], ChordResult { root: 0, chord_type: 15 }),
            (vec![64, 67, 71, 74, 77], ChordResult { root: 7, chord_type: 15 }),
            (vec![60, 66, 72], ChordResult { root: 0, chord_type: 0 }),
            (vec![60, 68, 75], ChordResult { root: 8, chord_type: 1 }),
            (vec![60, 64, 70], ChordResult { root: 0, chord_type: 0 }),
            (vec![60, 63, 67], ChordResult { root: 0, chord_type: 2 }),
            (vec![60, 63, 66, 69], ChordResult { root: 0, chord_type: 7 }),
            (vec![60, 66, 72, 78, 84], ChordResult { root: 0, chord_type: 0 }),
        ];

        let mut finder = ChordFinder::new();
        for (notes, expected) in cases {
            let result = finder.identify_chord(&notes);
            assert_eq!(
                result, expected,
                "notes {:?}: expected {:?}, got {:?}",
                notes, expected, result
            );
        }
    }
}
