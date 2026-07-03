//! Chord-name assembly.
//!
//! [`ChordBuilder`] takes the sorted intervals produced by
//! [`crate::chord::Chord`] and turns them into a chord symbol. It collects a
//! [`ChordQuality`] (major/minor/dominant/…), an optional [`Extension`]
//! (6, 7, 9, 11, 13), a [`Suspension`] (sus2/sus4) and any alterations or
//! added tones, then renders them via [`ChordBuilder::compile_name`].

use std::fmt::Display;
use crate::interval::Interval;
use crate::interval::Name::{Fifth, Fourth, Root, Second, Seventh, Sixth, Third, Ninth, Eleventh, Thirteenth, Octave};
use crate::interval::Quality::{Augmented, Diminished, Major, Minor, Perfect};
use ChordQuality as Q;
use Extension as E;
use Suspension as Sus;
use crate::chord_builder::Suspension::{Sus2, Sus4};

#[derive(PartialEq, Eq, Clone)]
pub(crate) struct ChordBuilder {
    quality: ChordQuality,
    extension: Option<Extension>,
    suspension: Suspension,
    alterations: Vec<String>,
    adds: Vec<String>,
}

impl ChordBuilder {

    pub(crate) fn build(intervals: &Vec<Interval>) -> ChordBuilder {
        if intervals.is_empty() {
            panic!("Chord::build called with no intervals");
        }
        if !intervals.is_sorted() {
            panic!("Chord::build called with unsorted intervals");
        }

        let mut desc = ChordBuilder { quality: Q::None, extension: None, suspension: Sus::None, alterations: vec![], adds: vec![] };
        let mut old_quality: ChordQuality = Q::None;

        for interval in intervals {
            match (interval.name, interval.quality) {
                // Root, octave and perfect fifth don't change the chord name
                (Root, Perfect) | (Octave, Perfect) | (Fifth, Perfect) => {}

                // Determine quality / suspension
                (Second, Major) => desc.suspension = Sus2,
                (Third, Major) => desc.quality = Q::Major,
                (Third, Minor) => desc.quality = Q::Minor,
                (Fourth, Perfect) => desc.suspension = Sus4,
                (Fifth, Diminished) => {
                    old_quality = desc.quality;
                    desc.quality = Q::Diminished
                },
                (Fifth, Augmented) => if desc.quality == Q::Major {
                    desc.quality = Q::Augmented;
                } else {
                    desc.alterations.push(interval.to_string());
                },

                (Seventh, Diminished) => desc.quality = Q::Diminished7,
                (Seventh, _) => if desc.quality == Q::Diminished {
                    desc.quality = old_quality;
                    desc.add_upper_structure(*interval);
                    desc.add_upper_structure(Fifth + Diminished);
                } else {
                    desc.add_upper_structure(*interval);
                },
                // Sixth through thirteenth: the natural quality becomes an
                // extension, any other quality becomes an alteration/addition.
                (Sixth | Ninth | Eleventh | Thirteenth, _) =>
                    desc.add_upper_structure(*interval),

                (_, _) => unreachable!("Illegal interval: {}", interval.verbose_name()), // Ideally, create_and_sort_intervals() should eliminate every other possibility
            }
        }
        desc
    }

    pub(crate) fn compile_name(&mut self) -> Result<String, String> {
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

    /// Add an upper-structure tone: a natural extension is registered via
    /// [`Self::set_extension`], anything else falls back to an alteration/add.
    fn add_upper_structure(&mut self, interval: Interval) {
        if !self.set_extension(interval) {
            self.set_alteration(interval);
        }
    }

    /// Register `extension` as a chord extension. Returns `false` if the
    /// interval isn't a natural extension, so the caller can treat it as an
    /// alteration instead.
    fn set_extension(&mut self, extension: Interval) -> bool {
        macro_rules! validate_descriptor {
            ($desc:expr, $interval:expr, $($variant:path),+ $(,)?) => {
                if !(vec![$($variant),+].contains(&$desc.quality) || $desc.suspension != Sus::None) {
                    panic!("Interval {} not compatible with quality {}", $interval, &$desc.quality)
                }
            };
        }

        match (extension.name, extension.quality) {
            (Sixth, Major) => {
                validate_descriptor!(self, Sixth + Major, Q::Major, Q::Minor);
                self.extension = Some(E::Sixth);
                true
            }
            (Seventh, Minor) => {
                validate_descriptor!(self, Seventh + Minor, Q::Major, Q::Minor);
                if self.quality == Q::Major {
                    self.quality = Q::Dominant
                }
                self.extension = Some(E::Seventh);
                true
            },
            (Seventh, Major) => {
                validate_descriptor!(self, Seventh + Major, Q::Major, Q::Minor);
                if self.quality == Q::Minor {
                    self.quality = Q::MinorMajor
                }
                self.extension = Some(E::Seventh);
                true
            },
            (Ninth, Major) => {
                validate_descriptor!(self, Ninth + Major, Q::Major, Q::Minor, Q::Dominant, Q::MinorMajor);
                if self.extension.is_some_and(|e| e == E::Sixth) {
                    self.extension = Some(E::SixNine);
                } else if self.extension.is_some_and(|e| e == E::Seventh) {
                    self.extension = Some(E::Ninth);
                } else {
                    self.adds.push(format!("ᵃᵈᵈ{}", E::Ninth.to_string()));
                }
                true
            },
            (Eleventh, Perfect) => {
                validate_descriptor!(self, Eleventh + Perfect, Q::Major, Q::Minor, Q::Dominant, Q::MinorMajor);
                if self.extension.is_some_and(|e| e as i32 >= 7 && e as i32 <=11) {
                    self.extension = Some(E::Eleventh);
                } else {
                    self.adds.push(format!("ᵃᵈᵈ{}", E::Eleventh.to_string()));
                }
                true
            },
            (Thirteenth, Major) => {
                validate_descriptor!(self, Thirteenth + Major, Q::Major, Q::Minor, Q::Dominant, Q::MinorMajor);
                if self.extension.is_some_and(|e| e as i32 >= 7 && e as i32 <=13) {
                    self.extension = Some(E::Thirteenth);
                } else {
                    self.adds.push(format!("ᵃᵈᵈ{}", E::Thirteenth.to_string()));
                }
                true
            },
            _ => false,
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
