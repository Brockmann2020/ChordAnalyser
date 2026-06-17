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
        let mut value = self.value as isize - rhs.value as isize;
        if value < 0 {
            value = -value;
        }
        
        let mut result = Interval::from_value(value);
        if result.value > 12 {
            result.shift_octave();
        }
        result
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