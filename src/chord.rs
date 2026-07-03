//! Chord recognition: turn a set of notes into a chord name.
//!
//! [`Chord::from_notes`] takes the first note as the root and works in two
//! stages:
//!
//! 1. `create_and_sort_intervals` builds the interval from the root to every
//!    note and then places each interval in its musically correct octave
//!    (a second becomes a ninth when a third is present, a diminished fifth
//!    becomes a sharp eleventh over a perfect fifth, and so on).
//! 2. [`ChordBuilder`] walks the sorted intervals and assembles the name from
//!    a quality (major/minor/dominant/…), an optional extension (6, 7, 9, 11,
//!    13), a suspension (sus2/sus4) and any alterations or added tones.
//!
//! The result is a [`Chord`] whose [`Display`] is the chord symbol, e.g. `C⁷`.

use std::collections::HashSet;
use std::fmt::{format, Display};
use std::path::Prefix;
use crate::interval::{Interval, Name, Quality};
use crate::interval::Name::{Fifth, Fourth, Root, Second, Seventh, Sixth, Third, Octave};
use crate::interval::Quality::{Augmented, Diminished, Major, Minor, Perfect};
use crate::note::{Letter, Note};
use crate::note::Accidental::Sharp;
use crate::chord_builder::ChordBuilder;

#[derive(Debug)]
pub struct Chord {
    name: String,
    notes: Vec<Note>,
    intervals: Vec<Interval>,
    //alternatives: Vec<SlashChord>,
}

#[derive(Debug)]
pub struct SlashChord {
    name: String,
    root: Note,
    intervals: Vec<Interval>,
    is_inversion: bool,
}

impl Chord {

    pub fn from_notes(notes: Vec<Note>) -> Result<Chord, String> {
        if notes.is_empty() {
            panic!("Chord::from_notes called with no notes");
        } else if notes.len() == 1 {
            panic!("Chord::from_notes called with only one note");
        }

        let intervals = Self::create_and_sort_intervals(&notes);
        let root = *notes.first().unwrap();
        let name = format!("{}{}", root.to_string(), ChordBuilder::build(&intervals).compile_name()?);
        let /*mut*/ alternatives: Vec<SlashChord> = Vec::new();

        /*
        let mut seen = HashSet::new();
        let mut sub_notes: Vec<Note> = notes.clone().into_iter().filter(|x| seen.insert(*x)).collect(); // Remove duplicate Notes

        for i in 0..notes.len() {
            let root = sub_notes.remove(0);
            let new_root = sub_notes.first().copied();
            let mut is_inversion = false;

            // Determine if new root is from the first three stacked triads
            if let Some(new_root) = new_root {
                match (new_root + root).name {
                    Third | Fifth | Seventh => is_inversion = true,
                    _ => ()
                }
            } else {
                break;
            }

            // If it's an inversion, the root note is part of the chord rather than just the name
            if is_inversion {
                sub_notes.push(root);
            }

            // Build SlashChord
            let sub_intervals = Self::create_and_sort_intervals(&sub_notes);
            let alt_name: String;
            if let Ok(sub_intervals) = sub_intervals {
                alt_name = Self::evaluate_name(new_root.unwrap(), &sub_intervals);
                alternatives.push(SlashChord {name: alt_name, root: new_root.unwrap(), intervals: sub_intervals, is_inversion})
            } else {
                // todo!("Do Something with the error...")
            }

            // Add the note back now for next evaluation
            if !is_inversion {
                sub_notes.push(root);
            }
        }*/

        // Build Chord
        Ok(Chord {name, notes, intervals, /*alternatives*/})
    }

    fn create_and_sort_intervals(notes: &Vec<Note>) -> Vec<Interval> {
        let root = *notes.first().unwrap();
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
        let has_p5 = interval_iter.iter().find(|interval| {**interval == Fifth + Perfect}).is_some();
        let mut is_diminished = false;

        for interval in intervals.iter_mut() {

            if interval.name == Root || interval.name == Octave || *interval == Fifth + Perfect {
                continue;
            }

            // Second becomes ninth if either a third is present or if the quality is minor
            if *interval == Second + Minor || *interval == Second + Major && third.is_some() {
                interval.shift_octave(); continue;
            }

            // Minor Third becomes Sharp Ninth if Major third is present -> Jimi Hendrix Chord
            if *interval == Third + Minor && third.is_some_and(|t|t.quality == Major){
                let str = format!("{}",third.unwrap().verbose_name());
                interval.shift_octave(); continue;
            }

            // Fourth becomes eleventh if third is present
            if interval.name == Fourth && third.is_some() {
                interval.shift_octave(); continue;
            }

            // Perfect fifth doesn't affect naming, flat five turns into sharp eleventh if perfect 5 is present.
            if *interval == Fifth + Diminished {
                if has_p5 {
                    interval.shift_octave();
                } else if third.is_some_and(|t|t.quality == Minor) && seventh.is_none() { // Minor third + flat five -> diminished chord
                    is_diminished = true;
                }
                continue;
            }

            // Minor sixth can be either a diminished seventh or an augmented five depending on the third
            if interval.name == Sixth {
                if interval.quality == Minor {
                    if let Some(third) = third {
                        match third.quality {
                            Major => *interval = Fifth + Augmented,
                            Minor => if sixth.is_some_and(|i|i.quality == Major) {
                                interval.shift_octave();
                            }
                            _ => unreachable!()
                        }
                    }
                    if let Some(seventh) = seventh {
                        match seventh.quality {
                            Major => {
                                if has_p5 {
                                    interval.shift_octave();
                                } else {
                                    *interval = Fifth + Augmented
                                }
                            }
                            Minor => interval.shift_octave(),
                            _ => unreachable!()
                        }
                    }
                } else if interval.quality == Major {
                    if is_diminished {
                        *interval = Seventh + Diminished
                    }
                }
                continue;
            }

            if interval.name == Seventh {
                if  interval.quality == Major && seventh.is_some_and(|i|i.quality == Minor) {
                    panic!("A chord can't have a Major and a Minor Seventh at the same time")
                }
                continue;
            }
        }
        intervals.sort();

        intervals
    }

    pub fn detailed_string(&self) -> String {
        fn vec_to_string<T: Display>(vec: &Vec<T>) -> String {format!(
            "{}",
            vec.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )}

        format!("name: {},\nnotes: {},\nintervals: {},", self.name, vec_to_string(&self.notes), vec_to_string(&self.intervals))
    }
}

impl Display for Chord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.name)
    }
}
