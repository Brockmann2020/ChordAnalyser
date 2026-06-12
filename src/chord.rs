use crate::interval::Interval;
use crate::interval::Name::{Fifth, Fourth, Root, Second, Seventh, Sixth, Third};
use crate::interval::Quality::{Diminished, Major, Minor, Perfect};
use crate::note::Note;

struct Chord {
    name: String,
    notes: Vec<Note>,
    intervals: Vec<Interval>,
}

impl Chord {

    fn from_notes(notes: Vec<Note>) -> Result<Chord, String> {
        if notes.is_empty() {
            panic!("Chord::from_notes called with no notes");
        } else if notes.len() == 1 {
            panic!("Chord::from_notes called with only one note");
        }

        let root = *notes.first().unwrap();
        let mut name: String = root.to_string();
        let mut intervals = Vec::new();
        intervals.push(Root.into());

        // Get all the intervals
        for note in notes.iter() {
            if *note == root {
                continue;
            }
            intervals.push(root + *note);
        }
        intervals.sort();


        // 1. Determine Right Octave of the Interval
        let interval_iter = intervals.clone();
        let third: Option<&Interval> = interval_iter.iter().find(|interval| {interval.name == Third});
        let sixth: Option<&Interval> = interval_iter.iter().find(|interval| {interval.name == Sixth});
        let seventh: Option<&Interval> = interval_iter.iter().find(|interval| {interval.name == Seventh});
        let has_p5 = interval_iter.iter().find(|interval| {interval.name == Seventh}).is_some();
        let mut is_diminished = false;

        for interval in intervals.iter_mut() {

            // Second becomes ninth if either a third is present or if the quality is minor
            if interval.name == Second && interval.quality == Minor || third.is_some() {
                interval.shift_octave();
            }

            // Minor Third becomes Sharp Ninth if Major third is present -> Jimi Hendrix Chord
            if interval.is(Third, Minor) && third.is_some_and(|t|t.quality == Minor){
                interval.shift_octave();
            }

            // Fourth becomes eleventh if third is present
            if interval.name == Fourth && third.is_some() {
                interval.shift_octave();
            }

            // Perfect fifth doesn't affect naming, flat five turns into sharp eleventh if perfect 5 is present.
            if interval.is(Fifth, Diminished) {
                if has_p5 {
                    interval.shift_octave();
                } else if third.is_some_and(|t|t.quality == Minor) { // Minor third + flat five -> diminished chord
                    is_diminished = true;
                }
            }

            if interval.name == Sixth {
                // in a diminished chord minor 6 turns into diminished 7
                if interval.quality == Minor && is_diminished {
                    *interval = Interval {value: 8, name: Seventh, quality: Diminished }; // direct initialization, because new would return flat 6 -> ugly workaround for now
                }
            }


        }

        // Evaluate chord name


        Ok(Chord {name, notes, intervals})
    }

    fn create_and_sort_intervals(notes: Vec<Note>) -> Vec<Interval> {
        let mut intervals = Vec::new();
        intervals
    }
}