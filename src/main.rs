use note::{Note};
use crate::note::Letter::{A, B, C, D, E, F, G};
use crate::note::Accidental::{Sharp, Flat, Natural};

mod note;
mod interval;
mod chord;

fn main() {
    println!("{}", F+Flat);
}
