use std::{collections::HashMap, fmt, io::BufRead};

use itertools::Itertools;
use owo_colors::{colors::*, OwoColorize, Rgb};

use crate::solver::solver17::parse::PIECES;

use self::parse::{Coord, Jet, Piece};

mod parse;

const CHAMBER_WIDTH: usize = 7;
const CHAMBER_WIDTH_MASK: u8 = 0b0111_1111;

#[derive(Debug, Default)]
pub struct Solver17;

impl super::ChallengeSolver for Solver17 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        17
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let target = 2022;
        let verbose_output = false;

        let input = input.lines().next().unwrap()?;
        let jets = Jet::parse_all(&input)?;
        let mut state = State::default();

        while state.piece_count != target {
            // New piece starts falling
            let piece = &PIECES[state.piece_count % PIECES.len()];
            state.curr.x = 2;
            state.curr.y = state.top + 3;

            if verbose_output {
                println!("== Piece {} begins falling ==", state.piece_count + 1);
                println!("{state}");
            }

            loop {
                // jet fires
                let jet = &jets[state.jet_count % jets.len()];
                let new_curr = match jet {
                    Jet::Left => (state.curr.x.saturating_sub(1), state.curr.y).into(),
                    Jet::Right => (state.curr.x + 1, state.curr.y).into(),
                };
                if state.is_new_curr_valid(&new_curr, piece) {
                    state.curr = new_curr;
                }
                state.jet_count += 1;

                if verbose_output {
                    println!("Jet of gas pushes piece {jet} :",);
                    println!("{state}");
                }

                // piece falls
                let new_curr = (state.curr.x, state.curr.y.saturating_sub(1)).into();
                if state.curr.y == 0 || !state.is_new_curr_valid(&new_curr, piece) {
                    break;
                }
                state.curr = new_curr;

                if verbose_output {
                    println!("Piece falls 1 unit:");
                    println!("{state}");
                }
            }

            // piece settles
            for offset in piece.coords {
                let Coord { x, y } = state.curr + *offset;

                while state.map.len() <= y {
                    state.map.push(0);
                    state.color_map.push([Rgb(255, 255, 255); CHAMBER_WIDTH]);
                }

                state.map[y] |= pack_x_coord(x);
                state.color_map[y][x] = piece.color;

                state.top = state.top.max(y + 1);
            }

            // prep for next iteration
            state.piece_count += 1;
            if verbose_output {
                println!();
            }
        }

        if verbose_output {
            println!();
        }
        println!("== Final tower height: {} ==", state.top);

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let target = 1_000_000_000_000;
        let verbose_output = false;

        let input = input.lines().next().unwrap()?;
        let jets = Jet::parse_all(&input)?;
        let mut state = State::default();

        state.seen.reserve(input.len() * jets.len());

        while state.piece_count != target {
            // New piece starts falling
            let piece = &PIECES[state.piece_count % PIECES.len()];
            state.curr.x = 2;
            state.curr.y = state.top + 3;

            if verbose_output {
                println!("== Piece {} begins falling ==", state.piece_count + 1);
                println!("{state}");
            }

            loop {
                // jet fires
                let jet = &jets[state.jet_count % jets.len()];
                let new_curr = match jet {
                    Jet::Left => (state.curr.x.saturating_sub(1), state.curr.y).into(),
                    Jet::Right => (state.curr.x + 1, state.curr.y).into(),
                };
                if state.is_new_curr_valid(&new_curr, piece) {
                    state.curr = new_curr;
                }
                state.jet_count += 1;

                if verbose_output {
                    println!("Jet of gas pushes piece {jet} :",);
                    println!("{state}");
                }

                // piece falls
                let new_curr = (state.curr.x, state.curr.y.saturating_sub(1)).into();
                if state.curr.y == 0 || !state.is_new_curr_valid(&new_curr, piece) {
                    break;
                }
                state.curr = new_curr;

                if verbose_output {
                    println!("Piece falls 1 unit:");
                    println!("{state}");
                }
            }

            // piece settles
            for offset in piece.coords {
                let Coord { x, y } = state.curr + *offset;

                while state.map.len() <= y {
                    state.map.push(0);
                    state.color_map.push([Rgb(255, 255, 255); CHAMBER_WIDTH]);
                }

                state.map[y] |= pack_x_coord(x);
                state.color_map[y][x] = piece.color;

                state.top = state.top.max(y + 1);
            }

            // Look for a cycle!
            if state.added_by_repeats == 0 {
                let key = SeenKey {
                    piece_index: state.piece_count % PIECES.len(),
                    jet_index: state.jet_count % jets.len(),
                };

                // At the third occurance of a key, the values in the seen map repeat.
                // This is because some of the first pieces will have hit the floor.
                // By the time a combination of pieces_idx, jets_idx comes around again, the fallen
                // blocks only interact with other blocks when falling. That is the first
                // repeatable cycle.
                if let Some(SeenState {
                    seen_key_count: 2,
                    piece_count: old_piece_count,
                    top: old_top,
                }) = state.seen.get(&key)
                {
                    // add as many pieces as possible without hitting the goal piece_count

                    println!("Cycle detected!");
                    println!("  current piece count = {}", state.piece_count);
                    println!("  current top         = {}", state.top);
                    println!("  old piece count     = {old_piece_count}");
                    println!("  old top             = {old_top}");

                    let delta_piece_count = state.piece_count - old_piece_count;
                    let delta_top = state.top - old_top;
                    println!("  delta piece count   = {delta_piece_count}");
                    println!("  delta top           = {delta_top}");

                    let repeats = (target - state.piece_count) / delta_piece_count;
                    println!("  repeats             = {repeats}");

                    println!(
                        "Adding {} pieces (for {} additional levels)",
                        repeats * delta_piece_count,
                        repeats * delta_top,
                    );

                    state.piece_count += repeats * delta_piece_count;
                    state.added_by_repeats += repeats * delta_top;

                    println!("  new piece count     = {}", state.piece_count);
                }

                // Update seen map
                state
                    .seen
                    .entry(key)
                    .and_modify(|seen_state| {
                        seen_state.seen_key_count += 1;
                        seen_state.piece_count = state.piece_count;
                        seen_state.top = state.top;
                    })
                    .or_insert(SeenState {
                        seen_key_count: 1,
                        piece_count: state.piece_count,
                        top: state.top,
                    });
            }

            // prep for next iteration
            state.piece_count += 1;
            if verbose_output {
                println!();
            }
        }

        if verbose_output {
            println!();
        }
        println!(
            "== Final tower height: {} ==",
            state.top + state.added_by_repeats
        );
        println!("({} levels added by repeats)", state.added_by_repeats);

        Ok(Box::new(()))
    }
}

#[derive(Default, Debug)]
struct State {
    /// Keeps track of how many jets have blown, in total
    jet_count: usize,
    /// Keeps track of how many pieces have started to fall
    piece_count: usize,
    /// Keeps track of how tall the tower currently is
    top: usize,
    /// Keeps track of where rocks have settled in each level of the tower.
    map: Vec<u8>,
    /// For colorizing the `map` when printing using [`owo_colors`].
    color_map: Vec<[Rgb; CHAMBER_WIDTH]>,
    /// The current origin of the currently-falling piece.
    curr: Coord,
    /// A map to keep track of seen combinations of `PIECES` and `jets` indices
    /// so that the simulation can be fast-forwarded.
    seen: HashMap<SeenKey, SeenState>,
    /// The number of pieces added by repeats.
    added_by_repeats: usize,
}

/// The combination of the index into `PIECES` and the index into `jets`.
#[derive(Debug, Default, PartialEq, Eq, Hash)]
struct SeenKey {
    piece_index: usize,
    jet_index: usize,
}

impl From<(usize, usize)> for SeenKey {
    fn from((piece_index, jet_index): (usize, usize)) -> Self {
        Self {
            piece_index,
            jet_index,
        }
    }
}

/// For keeping track of the state whenever unique combinations of [`SeenKey::piece_index`]
/// and [`SeenKey::jet_index`] arise.
#[derive(Default, Debug)]
struct SeenState {
    /// A count of how many times this [`SeenKey`] has been seen.
    seen_key_count: usize,
    /// The [`State::piece_count`] the last time this key was seen.
    piece_count: usize,
    /// The [`State::top`] the last time this key was seen.
    top: usize,
}

impl From<(usize, usize, usize)> for SeenState {
    fn from((seen_key_count, piece_count, top): (usize, usize, usize)) -> Self {
        Self {
            seen_key_count,
            piece_count,
            top,
        }
    }
}

impl State {
    /// Determine if a new `curr` coordinate would be valid if the state were
    /// to use it.
    fn is_new_curr_valid(&mut self, new_curr: &Coord, piece: &Piece) -> bool {
        piece.coords.iter().all(|offset| {
            let Coord { x, y } = new_curr + offset;

            while self.map.len() <= y {
                self.map.push(0);
                self.color_map.push([Rgb(255, 255, 255); CHAMBER_WIDTH]);
            }

            x < CHAMBER_WIDTH && self.map[y] & pack_x_coord(x) == 0
        })
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let curr_piece = &PIECES[self.piece_count % PIECES.len()];

        // Write the top of the display box
        writeln!(f, "{}", "╭───────╮".fg::<CustomColor<100, 100, 100>>())?;

        // Blocks are drawn using half-height unicode block characters so that
        // they appear square-ish in the terminal.
        // Unicode block characters for copy-pasting:
        // - block in upper half of cell:                               ▀
        // - block in lower half of cell:                               ▄
        // - 2 blocks, one in upper half and one in lower half of cell: █
        // - current piece's block in upper half of cell:               ⠛
        // - current piece's block in lower half of cell:               ⣤
        // - current piece's 2 blocks taking up entire cell:            ⣿
        // - current piece's block in upper half of cell, existing
        //   block in lower half of cell (an unfortunate compramise):   ░

        #[derive(Clone, Copy)]
        enum HalfCell {
            Air,
            Existing,
            Current,
        }

        // First, copy the current map and convert each cell to an enum value
        let mut print: Vec<Vec<_>> = self
            .map
            .iter()
            .zip(self.color_map.iter())
            .map(|(row, row_colors)| {
                (0..CHAMBER_WIDTH)
                    .into_iter()
                    .map(|x| {
                        if (row & pack_x_coord(x)) == 0 {
                            (HalfCell::Air, row_colors[x])
                        } else {
                            (HalfCell::Existing, row_colors[x])
                        }
                    })
                    .collect()
            })
            .collect();

        // Next, figure out how many rows we need to add to `print` to make
        // space for the current piece, add them, and add the current piece.
        let mut local_top = self.top;
        for offset in curr_piece.coords {
            let Coord { x, y } = self.curr + *offset;

            // Make sure we have enough air cells
            while print.len() <= y {
                print.push(vec![(HalfCell::Air, Rgb(255, 255, 255)); CHAMBER_WIDTH]);
            }

            // Convert the piece's coordinate to a piece cell
            print[y][x] = (HalfCell::Current, curr_piece.color);

            // Update the current local top count
            local_top = local_top.max(y + 1);
        }

        // Make sure we have an even amount of cells, adding a row of air if we don't
        if local_top % 2 != 0 {
            local_top += 1;
            print.push(vec![(HalfCell::Air, Rgb(255, 255, 255)); CHAMBER_WIDTH]);
        }

        // Actually print out the pieces. We use sliding windows of size 2 so that
        // we can represent pieces using half-height block and braille characters.
        let iter = (0..local_top).rev().tuples::<(_, _)>();

        for (top_half_row, bottom_half_row) in iter {
            let mut row_str = String::with_capacity(3 * 7); // 7 chars @ max 3 bytes per UTF-8 char

            for col in 0..CHAMBER_WIDTH {
                let (top_half, top_half_color) = &print[top_half_row][col];
                let (bottom_half, bottom_half_color) = &print[bottom_half_row][col];

                let chr = match (top_half, bottom_half) {
                    (HalfCell::Air, HalfCell::Air) => " ".to_string(),
                    (HalfCell::Air, HalfCell::Existing) => {
                        format!("{}", "▄".color(*bottom_half_color))
                    }
                    (HalfCell::Existing, HalfCell::Air) => {
                        format!("{}", "▀".color(*top_half_color))
                    }
                    (HalfCell::Existing, HalfCell::Existing) => {
                        if top_half_color == bottom_half_color {
                            format!("{}", "█".color(*bottom_half_color))
                        } else {
                            format!(
                                "{}",
                                "▄".color(*bottom_half_color).on_color(*top_half_color)
                            )
                        }
                    }
                    (HalfCell::Air, HalfCell::Current) => {
                        format!("{}", "⣤".color(*bottom_half_color))
                    }
                    (HalfCell::Current, HalfCell::Air) => format!("{}", "⠛".color(*top_half_color)),
                    (HalfCell::Current, HalfCell::Current) => {
                        if top_half_color == bottom_half_color {
                            format!("{}", "⣿".color(*bottom_half_color))
                        } else {
                            format!(
                                "{}",
                                "⣤".color(*bottom_half_color).on_color(*top_half_color)
                            )
                        }
                    }
                    (HalfCell::Existing, HalfCell::Current) => format!(
                        "{}",
                        "⣤".color(*bottom_half_color).on_color(*top_half_color)
                    ),
                    (HalfCell::Current, HalfCell::Existing) => format!(
                        "{}",
                        "⠛".color(*top_half_color).on_color(*bottom_half_color)
                    ),
                };

                row_str.push_str(&chr);
            }

            writeln!(f, "{0}{row_str}{0}", "│".fg::<CustomColor<100, 100, 100>>())?;
        }

        // Write the bottom of the display box
        write!(f, "{}", "╰───────╯".fg::<CustomColor<100, 100, 100>>())
    }
}

/// Packs an x coordinate into a u8
const fn pack_x_coord(x: usize) -> u8 {
    1_u8.wrapping_shl(x as _) & CHAMBER_WIDTH_MASK
}
