use note::{Note};
use crate::interval::Name::{Fifth, Seventh};
use crate::interval::Quality::{Augmented, Diminished, Perfect};
use crate::note::Letter::{A, B, C, D, E, F, G};
use crate::note::Accidental::{Sharp, Flat, Natural};
use crate::chord::Chord;

mod note;
mod interval;
mod chord;

fn main() {
    let notes: Vec<Note> = vec![C.into(), G.into(), E.into()];

    println!("{:?}", Chord::from_notes(notes).unwrap());
}
