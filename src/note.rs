use std::ops::{Add, Sub};

#[derive(Copy, Clone)]
struct Note {
    pub value: Value,
    pub accidental: Accidental,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Value {
    A=0, B=2, C=3, D=5, E=7, F=8, G=10
}

#[derive(Copy, Clone)]
enum Accidental {
    Natural=0, Sharp=1, Flat=-1
}

impl Add<usize> for Value {
    type Output = Note;
    fn add(self, rhs: usize) -> Note {
        Note::from_u8(self as usize + rhs)
    }
}

impl Sub<usize> for Value {
    type Output = Note;
    fn sub(self, rhs: usize) -> Note {
        Note::from_u8(self as usize - rhs)
    }
}

impl Note {
    fn new(value: Value, accidental: Accidental) -> Note {
        Note { value, accidental }
    }

    fn switch_accidental(&mut self) {
        match self.accidental {
            Accidental::Sharp => {
                let note = self.value - 2;
                self.value = note.value;
                self.accidental = note.accidental;
            },
            Accidental::Flat => {
                let note = self.value + 2;
                self.value = note.value;
                self.accidental = note.accidental;
            },
            _ => ()
        };
    }

    fn from_u8(value: usize) -> Note {
        match value {
            0 => Note { value: Value::A, accidental: Accidental::Natural },
            1 => Note { value: Value::A, accidental: Accidental::Sharp },
            2 => Note { value: Value::B, accidental: Accidental::Natural },
            3 => Note { value: Value::C, accidental: Accidental::Natural },
            4 => Note { value: Value::C, accidental: Accidental::Sharp },
            5 => Note { value: Value::D, accidental: Accidental::Natural },
            6 => Note { value: Value::D, accidental: Accidental::Sharp },
            7 => Note { value: Value::E, accidental: Accidental::Natural },
            8 => Note { value: Value::F, accidental: Accidental::Natural },
            10 => Note { value: Value::G, accidental: Accidental::Natural },
            11 => Note { value: Value::G, accidental: Accidental::Sharp },
            _ => panic!("Invalid value")
        }
    }
}