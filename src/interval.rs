use std::cmp::Ordering;
use std::fmt::{write, Display, Pointer};
use std::ops::Add;
use crate::interval::Quality::Diminished;
use crate::note::Note;

#[derive(Copy, Clone, Eq, Debug)]
pub struct Interval {
    pub value: usize,
    pub name: Name,
    pub quality: Quality
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Name {
    Root=0,
    Second=2,
    Third=4,
    Fourth=5,
    Fifth=7,
    Sixth=9,
    Seventh=11,
    Octave=12,
    Ninth=14,
    Eleventh=17,
    Thirteenth=20,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Quality {
    Perfect,
    Major,
    Minor,
    Augmented,
    Diminished,
}

impl Quality {
    pub fn safe_cast_to_isize(&self, name: Name) -> isize {
        if *self == Diminished {
            return match name {
                Name::Fourth => -1,
                Name::Fifth => -1,
                _ => -2
            }
        }
        (*self).into()
    }
}

/**
Diminished can have different delta values depending on the interval that it's applied to. Use save_cast_to_isize for accurate casting
*/
impl Into<isize> for Quality {
    fn into(self) -> isize {
        match self {
            Quality::Perfect => 0,
            Quality::Major => 0,
            Quality::Minor => -1,
            Quality::Augmented => 1,
            Quality::Diminished => -1
        }
    }
}

/**
Lets you use addition for Interval initialization.
*/
impl Add<Quality> for Name {
    type Output = Interval;
    fn add(self, other: Quality) -> Interval {
        let value = self as isize + other.safe_cast_to_isize(self);

        // This prevents panics and undefined behavior (diminished root or nonsense like that), although a more precise check and error handling should be implemented. For now, it should work in most scenarios (if used correctly).
        if value < 0 || value > 24 {
            return Interval::from_value(value);
        }

        Interval {value: value as usize, name: self, quality: other}
    }
}

impl Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            Quality::Perfect => "Perfect",
            Quality::Major => "Major",
            Quality::Minor => "Minor",
            Quality::Augmented => "Augmented",
            Diminished => "Diminished", 
        };
        write!(f, "{}", str)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match self {
            Name::Root => "Root",
            Name::Second => "Second",
            Name::Third => "Third",
            Name::Fourth => "Fourth",
            Name::Fifth => "Fifth",
            Name::Sixth => "Sixth",
            Name::Seventh => "Seventh",
            Name::Octave => "Octave",
            Name::Ninth => "Ninth",
            Name::Eleventh => "Eleventh",
            Name::Thirteenth => "Thirteenth",
        };
        write!(f, "{}", str)
    }
}

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.quality)
    }
}

impl PartialEq<Self> for Interval {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd<Self> for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Interval {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl From<Name> for Interval {
    fn from(name: Name) -> Self {
        Interval::from_value(name as isize)
    }
}

impl From<isize> for Interval {
    fn from(value: isize) -> Self {
        Interval::from_value(value)
    }
}

impl Interval {
    
    pub fn new(value: usize, name: Name, quality: Quality) -> Self {
        name + quality
    }
    
    pub fn shift_octave(&mut self) {
        match self.value {
            0..12 => *self = Interval::from_value((self.value + 12) as isize),
            _ => *self = Interval::from_value((self.value - 12) as isize)
        }
    }

    pub fn from_value(value: isize) -> Interval {
        let value = value.rem_euclid(24) as usize;
        let (name, quality) = match value {
            0       => (Name::Root,        Quality::Perfect),
            1       => (Name::Second,      Quality::Minor),
            2       => (Name::Second,      Quality::Major),
            3       => (Name::Third,       Quality::Minor),
            4 | 16  => (Name::Third,       Quality::Major),
            5       => (Name::Fourth,      Quality::Perfect),
            6 | 19  => (Name::Fifth,       Quality::Diminished),
            7       => (Name::Fifth,       Quality::Perfect),
            8       => (Name::Sixth,       Quality::Minor),
            9       => (Name::Sixth,       Quality::Major),
            10 | 22 => (Name::Seventh,     Quality::Minor),
            11 | 23 => (Name::Seventh,     Quality::Major),
            12 | 24 => (Name::Octave,      Quality::Perfect),
            13      => (Name::Ninth,       Quality::Minor),
            14      => (Name::Ninth,       Quality::Major),
            15      => (Name::Ninth,       Quality::Augmented),
            17      => (Name::Eleventh,    Quality::Perfect),
            18      => (Name::Eleventh,    Quality::Augmented),
            20      => (Name::Thirteenth,  Quality::Minor),
            21      => (Name::Thirteenth,  Quality::Major),
            _  => unreachable!()
        };
        Interval { value, name, quality }
    }
}