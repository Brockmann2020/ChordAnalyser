use std::collections::HashSet;
use crate::interval::{Interval, Name, Quality};
use crate::interval::Name::{Fifth, Fourth, Root, Second, Seventh, Sixth, Third, Thirteenth};
use crate::interval::Quality::{Augmented, Diminished, Major, Minor, Perfect};
use crate::note::{Letter, Note};

#[derive(Debug)]
pub struct Chord {
    name: String,
    notes: Vec<Note>,
    intervals: Vec<Interval>,
    alternatives: Vec<SlashChord>,
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

        let intervals = Self::create_and_sort_intervals(&notes).unwrap(); // Implement Error Handling later
        let root = *notes.first().unwrap();
        let name = Self::evaluate_name(root, &intervals);
        let mut alternatives: Vec<SlashChord> = Vec::new();

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
        Ok(Chord {name, notes, intervals, alternatives})
    }

    fn evaluate_name(root: Note, intervals: &Vec<Interval>) -> String {

        "Not implemented yet".to_string()
    }

    fn create_and_sort_intervals(notes: &Vec<Note>) -> Result<Vec<Interval>, String> {
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
        let has_p5 = interval_iter.iter().find(|interval| {interval.name == Seventh}).is_some();
        let mut is_diminished = false;

        for interval in intervals.iter_mut() {

            // Second becomes ninth if either a third is present or if the quality is minor
            if *interval == Second + Minor || *interval == Second + Major && third.is_some() {
                interval.shift_octave(); continue;
            }

            // Minor Third becomes Sharp Ninth if Major third is present -> Jimi Hendrix Chord
            if *interval == Third + Minor && third.is_some_and(|t|t.quality == Minor){
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
                } else if third.is_some_and(|t|t.quality == Minor) { // Minor third + flat five -> diminished chord
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
                if  interval.quality == Minor && seventh.is_some_and(|i|i.quality == Major) ||
                    interval.quality == Major && seventh.is_some_and(|i|i.quality == Minor) {
                    return Err("A chord can't have a Major and a Minor Seventh at the same time".to_string())
                }
                continue;
            }
        }

        Ok(intervals)
    }
}