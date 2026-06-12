use std::cmp::Ordering;
use std::ops::Add;
use crate::note::Note;

#[derive(Copy, Clone, Eq,)]
pub struct Interval {
    pub value: usize,
    pub name: Name,
    pub quality: Quality
}

#[derive(Copy, Clone, PartialEq, Eq,)]
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

#[derive(Copy, Clone, PartialEq, Eq,)]
pub enum Quality {
    Perfect,
    Major,
    Minor,
    Augmented,
    Diminished,
}

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

impl Add<Quality> for Name {
    type Output = Interval;
    
    fn add(self, other: Quality) -> Interval {
        Interval::from_value(self as isize + other as isize)
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

    pub fn is(&self, name: Name, quality: Quality) -> bool {
        self.name == name && self.quality == quality
    }

    pub fn shift_octave(&mut self) {
        match self.value {
            0..12 => *self = Interval::from_value((self.value + 12) as isize),
            _ => *self = Interval::from_value((self.value - 12) as isize)
        }
    }

    pub fn from_value(value: isize) -> Interval {
        let value = (value % 24) as usize;
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