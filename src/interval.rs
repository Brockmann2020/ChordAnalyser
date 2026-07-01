use std::cmp::Ordering;
use std::fmt::{write, Display, Pointer};
use std::ops::Add;
use crate::interval::Quality::{Augmented, Diminished, Major, Minor, Perfect};
use crate::interval::Name::{Fifth, Fourth, Root, Second, Seventh, Sixth, Third, Ninth, Eleventh, Thirteenth, Octave};
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

impl Name {
    pub fn as_super_script(&self) -> String {
        match self {
            Root | Octave => "".to_string(),
            Second => "²".to_string(),
            Third => "³".to_string(),
            Fourth => "⁴".to_string(),
            Fifth => "⁵".to_string(),
            Sixth => "⁶".to_string(),
            Seventh => "⁷".to_string(),
            Ninth => "⁹".to_string(),
            Eleventh => "¹¹".to_string(),
            Thirteenth => "¹³".to_string(),
        }
    }

    pub fn as_degree(&self) -> String {
        match self {
            Root => "1".to_string(),
            Second => "2".to_string(),
            Third => "3".to_string(),
            Fourth => "4".to_string(),
            Fifth => "5".to_string(),
            Sixth => "6".to_string(),
            Seventh => "7".to_string(),
            Octave => "8".to_string(),
            Ninth => "9".to_string(),
            Eleventh => "11".to_string(),
            Thirteenth => "13".to_string(),

        }
    }
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
            Perfect => 0,
            Major => 0,
            Minor => -1,
            Augmented => 1,
            Diminished => -1
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
        let quality: &str = match self {
            Perfect | Major => "",
            Minor | Diminished => "♭",
            Augmented => "♯",
        };
        write!(f, "{}", quality)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", (*self as usize).to_string())
    }
}

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.quality, self.name.as_degree())
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

    pub fn verbose_name(&self) -> String {
        let quality = match self.quality {
            Perfect => "Perfect",
            Major => "Major",
            Minor => "Minor",
            Augmented => "Augmented",
            Diminished => "Diminished",
        };

        let name = match self.name {
            Root => "Root",
            Second => "Second",
            Third => "Third",
            Fourth => "Fourth",
            Fifth => "Fifth",
            Sixth => "Sixth",
            Seventh => "Seventh",
            Octave => "Octave",
            Ninth => "Ninth",
            Eleventh => "Eleventh",
            Thirteenth => "Thirteenth",
        };

        if self.name == Root || self.name == Octave {
            return name.to_string()
        }

        format!("{} {}", quality, name)
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
            0       => (Root,       Perfect),
            1       => (Second,     Minor),
            2       => (Second,     Major),
            3       => (Third,      Minor),
            4 | 16  => (Third,      Major),
            5       => (Fourth,     Perfect),
            6 | 19  => (Fifth,      Diminished),
            7       => (Fifth,      Perfect),
            8       => (Sixth,      Minor),
            9       => (Sixth,      Major),
            10 | 22 => (Seventh,    Minor),
            11 | 23 => (Seventh,    Major),
            12 | 24 => (Octave,     Perfect),
            13      => (Ninth,      Minor),
            14      => (Ninth,      Major),
            15      => (Ninth,      Augmented),
            17      => (Eleventh,   Perfect),
            18      => (Eleventh,   Augmented),
            20      => (Thirteenth, Minor),
            21      => (Thirteenth, Major),
            _  => unreachable!()
        };
        Interval { value, name, quality }
    }

    pub fn as_super_script(&self) -> String {
        format!("{}{}", self.quality, self.name.as_super_script())
    }
}