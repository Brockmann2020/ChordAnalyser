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
use crate::interval::Name::{Fifth, Fourth, Root, Second, Seventh, Sixth, Third, Ninth, Eleventh, Thirteenth, Octave};
use crate::interval::Quality::{Augmented, Diminished, Major, Minor, Perfect};
use crate::note::{Letter, Note};
use ChordQuality as Q;
use Extension as E;
use Suspension as Sus;
use crate::chord::Suspension::{Sus2, Sus4};
use crate::note::Accidental::Sharp;

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

        // quick hack
        let tmp = Self::create_and_sort_intervals(&notes);
        let intervals = tmp.0;
        let root = *notes.first().unwrap();
        let name = format!("{}{}", root.to_string(), ChordBuilder::build(&intervals, tmp.1).compile_name()?);
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

    fn create_and_sort_intervals(notes: &Vec<Note>) -> (Vec<Interval>, bool) {
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

        (intervals, is_diminished)
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

#[derive(PartialEq, Eq, Clone)]
struct ChordBuilder {
    quality: ChordQuality,
    extension: Option<Extension>,
    suspension: Suspension,
    alterations: Vec<String>,
    adds: Vec<String>,
}

impl ChordBuilder {

    fn build(intervals: &Vec<Interval>, is_dim: bool) -> ChordBuilder {
        if intervals.is_empty() {
            panic!("Chord::build called with no intervals");
        }
        if !intervals.is_sorted() {
            panic!("Chord::build called with unsorted intervals");
        }

        let mut desc = ChordBuilder { quality: Q::None, extension: None, suspension: Sus::None, alterations: vec![], adds: vec![] };

        for interval in intervals {
            match (interval.name, interval.quality) {
                //Root and Octave do nothing
                (Root, Perfect) | (Octave, Perfect) => { },
                // Determine Quality
                (Second, Major) => desc.suspension = Sus2,
                (Third, Major) => desc.quality = Q::Major,
                (Third, Minor) => desc.quality = Q::Minor,
                (Fourth, Perfect) => desc.suspension = Sus4,
                (Fifth, Diminished) => {
                    if is_dim {
                        desc.quality = Q::Diminished
                    } else {
                        desc.alterations.push((Fifth + Diminished).as_super_script());
                    }
                },
                (Fifth, Perfect) => (), // Perfect Fifth doesn't change chord name if interval vector was assembled correctly
                (Fifth, Augmented) => {
                    if desc.quality == Q::Major {
                        desc.quality = Q::Augmented;
                    } else {
                        desc.alterations.push((Fifth + Augmented).to_string())
                    }
                },
                // Determine Extensions / Alterations
                (Sixth, Minor) => desc.set_alteration(Sixth + Minor),
                (Sixth, Major) => desc.set_extension(Sixth + Major),
                (Seventh, Diminished) => desc.quality = Q::Diminished7,
                (Seventh, Minor) => desc.set_extension(Seventh + Minor),
                (Seventh, Major) => desc.set_extension(Seventh + Major),
                (Ninth, Major) => desc.set_extension(Ninth + Major),
                (Ninth, q) => desc.set_alteration(Ninth + q),
                (Eleventh, Perfect) => desc.set_extension(Eleventh + Perfect),
                (Eleventh, q) => desc.set_alteration(Eleventh + q),
                (Thirteenth, Major) => desc.set_extension(Thirteenth + Major),
                (Thirteenth, q) => desc.set_alteration(Thirteenth + q),

                (_, _) => unreachable!("Illegal interval: {}", interval.verbose_name()), // Ideally, create_and_sort_intervals() should eliminate every other possibility
            }
        }
        desc
    }

    fn compile_name(&mut self) -> Result<String, String> {
        if self.quality == Q::None {
            return Err("Quality is None. Was build() called?".to_string())
        }
        if self.extension.is_some() && self.adds.len() > 0 {
            return Err("A chord has either an extension or additions. It cannot have both.".to_string());
        }
        fn alt_or_adds(alt: &mut Vec<String>, adds: &mut Vec<String>) -> String {
            fn format_vector_or_default<T: Display>(vec: &Vec<T>) -> String {
                if vec.is_empty() {
                    return String::new();
                }
                format!(
                    "⁽{}⁾",
                    vec.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("𝇅 ")
                )
            }

            if !alt.is_empty() {
                return format_vector_or_default(&alt);
            }
            format_vector_or_default(&adds)
        }

        // quality + sus + alt or adds
        Ok(format!("{}{}{}", self.format_quality(),  self.suspension.to_string(), alt_or_adds(&mut self.alterations, &mut self.adds)))
    }

    fn set_extension(&mut self, extension: Interval) {
        macro_rules! validate_descriptor {
            ($desc:expr, $($variant:path),+ $(,)?) => {
                if !(vec![$($variant),+].contains(&$desc.quality) || $desc.suspension != Sus::None) {
                    panic!("Interval not compatible with quality") //todo: better error
                }
            };
        }

        match (extension.name, extension.quality) {
            (Sixth, Major) => {
                validate_descriptor!(self, Q::Major, Q::Minor);
                self.extension = Some(E::Sixth)
            }
            (Seventh, Minor) => {
                validate_descriptor!(self, Q::Major, Q::Minor);
                if self.quality == Q::Major {
                    self.quality = Q::Dominant
                }
                self.extension = Some(E::Seventh)
            },
            (Seventh, Major) => {
                validate_descriptor!(self, Q::Major, Q::Minor);
                if self.quality == Q::Minor {
                    self.quality = Q::MinorMajor
                }
                self.extension = Some(E::Seventh)
            },
            (Ninth, Major) => {
                validate_descriptor!(self, Q::Major, Q::Minor, Q::Dominant, Q::MinorMajor);
                if self.extension.is_some_and(|e| e == E::Sixth) {
                    self.extension = Some(E::SixNine);
                } else if self.extension.is_some_and(|e| e == E::Seventh) {
                    self.extension = Some(E::Ninth);
                } else {
                    self.adds.push(format!("ᵃᵈᵈ{}", E::Ninth.to_string()));
                }
            },
            (Eleventh, Perfect) => {
                validate_descriptor!(self, Q::Major, Q::Minor, Q::Dominant, Q::MinorMajor);
                if self.extension.is_some_and(|e| e as i32 >= 7 && e as i32 <=11) {
                    self.extension = Some(E::Eleventh);
                } else {
                    self.adds.push(format!("ᵃᵈᵈ{}", E::Eleventh.to_string()));
                }
            },
            (Thirteenth, Major) => {
                validate_descriptor!(self, Q::Major, Q::Minor, Q::Dominant, Q::MinorMajor);
                if self.extension.is_some_and(|e| e as i32 >= 7 && e as i32 <=13) {
                    self.extension = Some(E::Thirteenth);
                } else {
                    self.adds.push(format!("ᵃᵈᵈ{}", E::Thirteenth.to_string()));
                }
            },
            (n, q) => panic!("Illegal extension interval: {}", n + q)
        }
    }

    fn set_alteration(&mut self, alteration: Interval) {
        if self.extension.is_some() {
            self.alterations.push(alteration.as_super_script())
        } else {
            self.adds.push(format!("ᵃᵈᵈ{}", alteration.as_super_script()));
        }
    }

    fn format_quality(&self) -> String {
        fn display_or_default(extension: Option<Extension>) -> String {
            if let Some(extension) = extension {
                return extension.to_string();
            }
            "".to_string()
        }

        match self.quality {
            Q::None => panic!("Quality cannot be None"),
            Q::Major => {
                if let Some(extension) = self.extension {
                    format!("maj{}", extension)
                } else {
                    "".to_string()
                }
            },
            Q::Minor => format!("m{}", display_or_default(self.extension)),
            Q::Dominant => format!("{}", display_or_default(self.extension)),
            Q::MinorMajor => format!("m(maj{})", display_or_default(self.extension)),
            Q::Augmented => format!("aug{}", display_or_default(self.extension)),
            Q::Diminished => format!("dim{}", display_or_default(self.extension)),
            Q::Diminished7 => "dim7".to_string(),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
enum Extension {
    Sixth=6,
    Seventh=7,
    Ninth=9,
    SixNine=69,
    Eleventh=11,
    Thirteenth=13,
}

impl Display for Extension {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        /*let number = *self as usize;
        if number == 69 {
            write!(fmt, "6/9")
        } else {
            write!(fmt, "{}", number)
        }*/
        let str = match self {
            E::Sixth => "⁶".to_string(),
            E::Seventh => "⁷".to_string(),
            E::Ninth => "⁹".to_string(),
            E::SixNine => "⁶ᐟ⁹".to_string(),
            E::Eleventh => "¹¹".to_string(),
            E::Thirteenth => "¹³".to_string(),
        };
        write!(fmt, "{}", str)
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum Suspension {
    None,
    Sus2,
    Sus4,
}

impl Display for Suspension {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let str = match self {
            Sus::None => String::new(),
            Sus2 => "sus2".to_string(),
            Sus4 => "sus4".to_string(),
        };
        write!(fmt, "{}", str)
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum ChordQuality {
    None,
    Major,
    Minor,
    Dominant,
    MinorMajor,
    Augmented,
    Diminished,
    Diminished7,
}

impl Display for ChordQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Q::None => "None",
            Q::Major => "Major",
            Q::Minor => "Minor",
            Q::Dominant => "Dominant",
            Q::MinorMajor => "MinorMajor",
            Q::Augmented => "Augmented",
            Q::Diminished => "Diminished",
            Q::Diminished7 => "Diminished",
        };
        write!(f, "{}", str)
    }
}

