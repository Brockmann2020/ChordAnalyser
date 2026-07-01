# ChordAnalyser

A small command-line tool written in Rust that names a chord from a set of
notes. You give it the notes you play, it tells you the chord — e.g.
`C E G Bb → C⁷`.

## Build & Run

Requires a Rust toolchain (edition 2024).

```bash
# Interactive prompt
cargo run

# Analyse a single chord and exit
cargo run -- C E G Bb

# Same, but print the detailed view
cargo run -- -d C E G Bb
```

After a `cargo build --release` the binary lives at
`target/release/ChordAnalyser` and can be called directly.

## Entering notes

- Separate notes with **spaces or commas**: `C E G Bb` or `C, E, G, Bb`.
- Letters `A`–`G` (case-insensitive). Note: this uses the **English**
  convention, so `B` is the note a semitone below C, written `B` (not `H`),
  and `Bb`/`B♭` is a whole tone below.
- Accidentals: `#` or `♯` for sharp, `b` or `♭` for flat — e.g. `F#`, `Eb`.
- The **first note is the root**; it determines the chord's name.
- At least two notes are required.

### Interactive commands

| Input            | Effect                                        |
|------------------|-----------------------------------------------|
| `C E G Bb`       | Analyse and print the chord name              |
| `detail`         | Toggle the detailed view (name + notes + intervals) |
| `quit` / `exit`  | Leave the program (empty line or Ctrl+Z also exit) |

## Examples

```text
notes> C E G Bb
  => C⁷
notes> C E G B
  => Cmaj⁷
notes> C E G D
  => C⁽ᵃᵈᵈ⁹⁾
notes> C Eb G Bb
  => Cm⁷
notes> A C E G
  => Am⁷
```

Detailed view:

```text
notes> detail
  detailed view on
notes> C E G Bb
  name: C⁷,
  notes: C, E, G, B♭,
  intervals: 1, 3, 5, ♭7,
```

## How it works

The program is organised into three modules (see the rustdoc comments in each
file, or run `cargo doc --open`):

- **`note`** — note letters, accidentals and their semitone values, plus
  parsing of tokens like `Bb` and `F#`. The interval between two notes is
  measured *upward* from the root modulo the octave, so a note whose raw pitch
  falls below the root (e.g. B♭ under C) is correctly read as a minor seventh
  rather than a major second.
- **`interval`** — intervals as a diatonic name (third, fifth, seventh, …)
  plus a quality (perfect, major, minor, augmented, diminished), together with
  semitone arithmetic and octave shifting.
- **`chord`** — collects the intervals from the root to each note, places each
  in its correct octave based on musical context, and builds the final chord
  symbol from a quality, an optional extension (6/7/9/11/13), a suspension
  (sus2/sus4) and any alterations or added tones.

The overall flow is:

```
notes → parse → intervals from root → octave placement → chord name
```

## Limitations

- Inversions / slash chords are not reported (the scaffolding exists but is
  not yet serviceable).
- Not all combinations of notes are supported yet. More obscure random sets of notes cannot be evaluated.
```
