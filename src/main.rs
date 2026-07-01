//! # ChordAnalyser
//!
//! A small command-line tool that names a chord from a set of notes.
//!
//! Notes are read from the console (or from command-line arguments), parsed
//! into [`Note`]s, and handed to [`Chord::from_notes`], which computes the
//! intervals relative to the first note (the root) and derives a chord name
//! such as `C⁷`, `Cmaj⁷` or `Cm⁷⁽♭⁵⁾`.
//!
//! ## Usage
//!
//! ```text
//! cargo run                    # interactive prompt
//! cargo run -- C E G Bb        # analyse a single chord and exit
//! cargo run -- -d C E G Bb     # same, with the detailed view
//! ```
//!
//! In the interactive prompt the command `detail` toggles the detailed view
//! (name + notes + intervals) and `quit` exits.
//!
//! ## Module layout
//!
//! - [`note`]   — note letters, accidentals, pitch values and parsing.
//! - [`interval`] — intervals (name + quality) and semitone arithmetic.
//! - [`chord`]  — interval collection, octave placement and name building.

use std::io::{self, Write};
use note::Note;
use crate::chord::Chord;

mod note;
mod interval;
mod chord;

/// Split an input line into note tokens (separated by whitespace or commas)
/// and parse each one into a `Note`.
fn parse_notes(input: &str) -> Result<Vec<Note>, String> {
    input
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|t| !t.is_empty())
        .map(Note::parse)
        .collect()
}

/// Parse the input, build the chord and print the result. Never panics out:
/// chord building can panic on musically impossible input, so it is caught.
/// When `detailed` is set, the chord's notes and intervals are printed too.
fn analyze(input: &str, detailed: bool) {
    let notes = match parse_notes(input) {
        Ok(notes) => notes,
        Err(e) => {
            println!("  error: {}", e);
            return;
        }
    };

    if notes.len() < 2 {
        println!("  please enter at least 2 notes");
        return;
    }

    let result = std::panic::catch_unwind(|| Chord::from_notes(notes));
    match result {
        Ok(Ok(chord)) => {
            if detailed {
                println!("{}", indent(&chord.detailed_string()));
            } else {
                println!("  => {}", chord);
            }
        }
        Ok(Err(e)) => println!("  error: {}", e),
        Err(_) => println!("  error: these notes don't form a nameable chord"),
    }
}

/// Indent every line of a multi-line string by two spaces.
fn indent(text: &str) -> String {
    text.lines()
        .map(|line| format!("  {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() {
    // Keep the interactive loop clean: swallow the default panic backtrace,
    // catch_unwind in analyze() turns panics into a friendly message instead.
    std::panic::set_hook(Box::new(|_| {}));

    // Non-interactive mode: notes passed as command-line arguments.
    // A leading -d / --detailed flag prints the detailed chord view.
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let detailed_flag = args.first().is_some_and(|a| a == "-d" || a == "--detailed");
    if detailed_flag {
        args.remove(0);
    }
    if !args.is_empty() {
        analyze(&args.join(" "), detailed_flag);
        return;
    }

    // Interactive mode.
    let mut detailed = detailed_flag;
    println!("ChordAnalyser");
    println!("Enter notes separated by spaces or commas, e.g.  C E G Bb");
    println!("Accidentals: # (sharp) or b (flat), e.g.  F#  Eb");
    println!("Commands: 'detail' toggles the detailed view, 'quit' exits.\n");

    let stdin = io::stdin();
    loop {
        print!("notes> ");
        io::stdout().flush().ok();

        let mut line = String::new();
        if stdin.read_line(&mut line).unwrap_or(0) == 0 {
            break; // EOF (Ctrl+Z / Ctrl+D)
        }

        let line = line.trim();
        if line.is_empty()
            || line.eq_ignore_ascii_case("quit")
            || line.eq_ignore_ascii_case("exit")
        {
            break;
        }

        if line.eq_ignore_ascii_case("detail") || line.eq_ignore_ascii_case("detailed") {
            detailed = !detailed;
            println!("  detailed view {}", if detailed { "on" } else { "off" });
            continue;
        }

        analyze(line, detailed);
    }
}
