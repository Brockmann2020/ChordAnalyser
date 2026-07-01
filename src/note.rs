//! Notes: pitch letters, accidentals and their semitone values.
//!
//! A [`Note`] carries a raw semitone `value` (letter + accidental) together
//! with its spelling ([`Letter`] + [`Accidental`]), so that enharmonically
//! equal notes can still be distinguished when a chord is named.
//!
//! The [`Add`] implementation `Note + Note` yields the [`Interval`] measured
//! *upward* from the left note to the right note, modulo the octave. Measuring
//! upward (rather than as an absolute difference) is what makes a note below
//! the root — e.g. B♭ under C — resolve to a minor seventh instead of a
//! major second.

use std::arch::x86_64::__cpuid;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::write;
use std::ops::{Add, Sub};
use crate::interval::Interval;
use crate::note::Accidental::Natural;

#[derive(Copy, Clone, Eq, Hash, Debug)]
pub(crate) struct Note {
    pub value: u8,
    pub letter: Letter,
    pub accidental: Accidental,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Letter {
    A=0, B=2, C=3, D=5, E=7, F=8, G=10
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum Accidental {
    Natural=0, Sharp=1, Flat=-1
}

impl Add<Accidental> for Letter {
    type Output = Note;
    fn add(self, rhs: Accidental) -> Note {
        let value = self as isize + rhs as isize;
        let accidental = match rhs {
            Accidental::Natural => Accidental::Sharp,
            _ => rhs
        };
        Note::from_value(value, accidental)
    }
}

impl Letter {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Letter::A => "A",
            Letter::B => "B",
            Letter::C => "C",
            Letter::D => "D",
            Letter::E => "E",
            Letter::F => "F",
            Letter::G => "G",
        }
    }
}

impl PartialEq<Self> for Note {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd<Self> for Note {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Note {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl Add<usize> for Note {
    type Output = Note;
    fn add(self, rhs: usize) -> Self::Output {
        let value = self.value as isize + rhs as isize;
        let accidental = match self.accidental {
            Accidental::Natural => Accidental::Sharp,
            _ => self.accidental
        };
        Note::from_value(value, accidental)
    }
}

impl Add<Note> for Note {
    type Output = Interval;

    fn add(self, rhs: Note) -> Self::Output {
        let value = (rhs.value as isize - self.value as isize).rem_euclid(12);
        Interval::from_value(value)
    }
}

impl From<Letter> for Note {
    fn from(value: Letter) -> Self {
        value + Natural
    }
}

impl Note {
    fn new(letter: Letter, accidental: Accidental) -> Note {
        letter + accidental
    }

    /// Parse a single note token like `C`, `Bb`, `F#`, `Eb`, `A♯`.
    /// The first character is the letter (case-insensitive), an optional second
    /// character is the accidental: `#`/`♯` for sharp, `b`/`♭` for flat.
    pub fn parse(token: &str) -> Result<Note, String> {
        let token = token.trim();
        let mut chars = token.chars();

        let letter = match chars.next() {
            Some(c) => match c.to_ascii_uppercase() {
                'A' => Letter::A,
                'B' => Letter::B,
                'C' => Letter::C,
                'D' => Letter::D,
                'E' => Letter::E,
                'F' => Letter::F,
                'G' => Letter::G,
                other => return Err(format!("'{}' is not a valid note letter (A-G)", other)),
            },
            None => return Err("empty note".to_string()),
        };

        let accidental = match chars.next() {
            None => Accidental::Natural,
            Some('#') | Some('\u{266f}') => Accidental::Sharp,
            Some('b') | Some('\u{266d}') => Accidental::Flat,
            Some(other) => return Err(format!("'{}' is not a valid accidental (# or b)", other)),
        };

        if chars.next().is_some() {
            return Err(format!("'{}' is not a valid note", token));
        }

        Ok(letter + accidental)
    }

    fn switch_accidental(&mut self) {
        let new_accidental = match self.accidental {
            Accidental::Sharp => Accidental::Flat,
            Accidental::Flat => Accidental::Sharp,
            Accidental::Natural => Accidental::Natural
        };

        if new_accidental == Accidental::Natural {
            return ()
        }

        *self = Self::from_value(self.value as isize, new_accidental);
    }

    fn from_value(value: isize, preferred_accidental: Accidental) -> Note {
        let semitone = value.rem_euclid(12);
        match (semitone, preferred_accidental) {
            (0,  _)                => Note { value: value as u8, letter: Letter::A, accidental: Accidental::Natural },
            (1,  Accidental::Flat) => Note { value: value as u8, letter: Letter::B, accidental: Accidental::Flat },
            (1,  _)                => Note { value: value as u8, letter: Letter::A, accidental: Accidental::Sharp },
            (2,  _)                => Note { value: value as u8, letter: Letter::B, accidental: Accidental::Natural },
            (3,  _)                => Note { value: value as u8, letter: Letter::C, accidental: Accidental::Natural },
            (4,  Accidental::Flat) => Note { value: value as u8, letter: Letter::D, accidental: Accidental::Flat },
            (4,  _)                => Note { value: value as u8, letter: Letter::C, accidental: Accidental::Sharp },
            (5,  _)                => Note { value: value as u8, letter: Letter::D, accidental: Accidental::Natural },
            (6,  Accidental::Flat) => Note { value: value as u8, letter: Letter::E, accidental: Accidental::Flat },
            (6,  _)                => Note { value: value as u8, letter: Letter::D, accidental: Accidental::Sharp },
            (7,  _)                => Note { value: value as u8, letter: Letter::E, accidental: Accidental::Natural },
            (8,  _)                => Note { value: value as u8, letter: Letter::F, accidental: Accidental::Natural },
            (9,  Accidental::Flat) => Note { value: value as u8, letter: Letter::G, accidental: Accidental::Flat },
            (9,  _)                => Note { value: value as u8, letter: Letter::F, accidental: Accidental::Sharp },
            (10, _)                => Note { value: value as u8, letter: Letter::G, accidental: Accidental::Natural },
            (11, Accidental::Flat) => Note { value: value as u8, letter: Letter::A, accidental: Accidental::Flat },
            (11, _)                => Note { value: value as u8, letter: Letter::G, accidental: Accidental::Sharp },
            _                      => unreachable!()
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let accidental = match self.accidental {
            Accidental::Flat => "♭",
            Accidental::Natural => "",
            Accidental::Sharp => "♯",
        };
        write!(f, "{}{}", self.letter.as_str(), accidental)
    }
}