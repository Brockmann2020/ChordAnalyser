use std::collections::HashSet;
use std::fmt::Display;
use std::path::Prefix;
use crate::interval::{Interval, Name, Quality};
use crate::interval::Name::{Fifth, Fourth, Root, Second, Seventh, Sixth, Third, Ninth, Eleventh, Thirteenth, Octave};
use crate::interval::Quality::{Augmented, Diminished, Major, Minor, Perfect};
use crate::note::{Letter, Note};
use ChordQuality as Q;
use crate::chord::Suspension::{Sus2, Sus4};

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
        Ok(Chord {name, notes, intervals, alternatives})
    }

    fn evaluate_name(root: Note, intervals: &Vec<Interval>) -> String {


        let mut name: String = root.to_string();
        let mut quality: Option<ChordQuality> = None;
        let mut alterations: Vec<&str> = Vec::new();
        let mut adds: Vec<&str> = Vec::new();
        let mut suspension: Option<Suspension> = None;

        for interval in intervals {
            match (interval.name, interval.quality) {
                // Determine Quality
                (Second, Major) => {
                    quality = Some(Q::Sus);
                    suspension = Some(Sus2);
                },
                (Third, Major) => quality = Some(Q::Major(None)),
                (Third, Minor) => quality = Some(Q::Minor(None)),
                (Fourth, Perfect) => {
                    quality = Some(Q::Sus);
                    suspension = Some(Sus4)
                },
                (Fifth, Diminished) => quality = Some(Q::Diminished),
                (Fifth, Perfect) => (), // Perfect Fifth doesn't change chord name if vector was assembled correctly
                (Fifth, Augmented) => {
                    if quality.is_some_and(|q| q == Q::Major(None)) {
                        quality = Some(Q::Augmented);
                    } else {
                        alterations.push("♯5")
                    }
                },
                (Sixth, Minor) => alterations.push("♭6"),
                (Sixth, Major) => {
                    if let Some(q) = quality {
                        match q {
                            Q::Minor(None) => quality = Some(Q::Minor(Some(Sixth))),
                            Q::Major(None) | Q::Sus => quality = Some(Q::Major(Some(Sixth))),
                            q => unreachable!("6 is not allowed if quality is {}", q)
                        }
                    } else {
                        panic!("No third") //todo: add no third alteration
                    }
                },
                (Seventh, Diminished) => quality = Some(Q::Diminished7),
                (Seventh, Minor) => {
                    if let Some(q) = quality {
                        match q {
                            Q::Minor(None) => quality = Some(Q::Minor(Some(Seventh))),
                            Q::Major(None) | Q::Sus => quality = Some(Q::Dominant(Seventh)),
                            q => unreachable!("min7 is not allowed if quality is {}", q)
                        }
                    } else {
                        panic!("No third") //todo: add no third alteration
                    }
                },
                (Seventh, Major) => {
                    if let Some(q) = quality {
                        match q {
                            Q::Minor(None) => quality = Some(Q::MinorMajor(Seventh)),
                            Q::Major(None) | Q::Sus => quality = Some(Q::Major(Some(Seventh))),
                            q => unreachable!("maj7 not allowed if quality is {}", q)
                        }
                    } else {
                        panic!("No third") //todo: add no third alteration
                    }
                }
                (Octave, _) => { /* Octave does nothing */ },
                (Ninth, Major) => {
                    if let Some(q) = quality {
                        match q {
                            Q::Major(Some(extension)) | Q::Minor(Some(extension)) => {

                            }
                            Q::Dominant(_) | Q::MinorMajor(_) => {}
                            Q::Major(None) | Q::Minor(None) => adds.push("9"),
                            q => unreachable!("maj7 not allowed if quality is {}", q)
                        }
                    }
                },
                (Ninth, _) => {  },


                (_, _) => unreachable!("Illegal interval: {}", interval), // Ideally, create_and_sort_intervals() should eliminate every other possibility
            }
        }

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
                if  interval.quality == Major && seventh.is_some_and(|i|i.quality == Minor) {
                    return Err("A chord can't have a Major and a Minor Seventh at the same time".to_string())
                }
                continue;
            }
        }
        intervals.sort();

        Ok(intervals)
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum Suspension {
    Sus2,
    Sus4,
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum ChordQuality {
    Major(Option<Name>),
    Minor(Option<Name>),
    Dominant(Name),
    MinorMajor(Name),
    Augmented,
    Diminished,
    Diminished7,
    SixNine,
    Sus, // Might not be needed and replaced by None, but for now I believe it reduces risk of unforeseen errors
}

impl Display for ChordQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        macro_rules! format_extension {
            // Format for dominant extension
            ($extension:expr) => {{
                match $extension {
                    Seventh | Ninth | Eleventh | Thirteenth => $extension.to_string(),
                    name => panic!("Illegal extension name: name={}", name),
                }
            }};

            // Format for dominant, m(maj) extension
            ($extension:expr, $template:expr) => {{
                match $extension {
                    Seventh | Ninth | Eleventh | Thirteenth => {
                        $template.replace("{}", &$extension.to_string())
                    }
                    name => panic!("Illegal extension name: name={}", name),
                }
            }};

            // Format for min, maj extension
            ($extension:expr, $prefix:expr, $alt:expr) => {{
                if let Some(extension) = $extension {
                    match extension {
                        Sixth => format!("{}{}", $alt, extension.to_string()),
                        Seventh | Ninth | Eleventh | Thirteenth => format!("{}{}", $prefix, extension),
                        // If a false extension is used the resulting error cannot be recovered
                        name => panic!("Illegal extension name: name={}", name),
                    }
                } else {
                    $alt.to_string()
                }
            }};
        }

        let str: String = match self {
            Q::Major(extension) => format_extension!(extension, "maj", ""),
            Q::Minor(extension) => format_extension!(extension, "m", "m"),
            Q::Dominant(extension) => format_extension!(extension),
            Q::MinorMajor(extension) => format_extension!(extension, "m(maj{})"),
            Q::Augmented => "aug".to_string(),
            Q::Diminished => "dim".to_string(),
            Q::Diminished7 => "dim7".to_string(),
            Q::SixNine => "6/9".to_string(),
            Q::Sus => "".to_string(),
        };

        write!(f, "{}", str)
    }
}