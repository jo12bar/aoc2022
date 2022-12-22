use std::{
    fs::File,
    io::{BufReader, Read},
};

use color_eyre::eyre::Context;

use crate::grid::{Grid, GridCoord};

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver08;

impl ChallengeSolver for Solver08 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        8
    }

    fn solve_a(&mut self, mut input: BufReader<File>) -> color_eyre::Result<()> {
        let mut grid = String::new();
        input
            .read_to_string(&mut grid)
            .wrap_err("Could not read input file")?;

        let grid = parse_grid(&grid).wrap_err("Could not parse grid")?;

        let all_coords = (0..grid.height())
            .into_iter()
            .flat_map(|y| (0..grid.width()).map(move |x| GridCoord::from((x, y))));

        let num_visible_cells = all_coords
            .filter(|&coord| {
                let coord_height = grid.cell(coord).unwrap();
                let deltas: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

                deltas.iter().any(|&(dx, dy)| {
                    let mut cells_in_line = iter_trees_in_dir(&grid, coord, (dx, dy));
                    cells_in_line.all(|height| height < coord_height)
                })
            })
            .count();

        println!("Number of visible trees: {num_visible_cells}");

        Ok(())
    }

    fn solve_b(&mut self, mut input: BufReader<File>) -> color_eyre::Result<()> {
        let mut grid = String::new();
        input
            .read_to_string(&mut grid)
            .wrap_err("Could not read input file")?;

        let grid = parse_grid(&grid).wrap_err("Could not parse grid")?;

        let all_coords = (0..grid.height())
            .into_iter()
            .flat_map(|y| (0..grid.width()).map(move |x| GridCoord::from((x, y))));

        let (best_place, best_score) = all_coords
            .map(|coord| (coord, scenic_score(&grid, coord)))
            .max_by_key(|(_, score)| *score)
            .unwrap();

        println!("Best location: {best_place:?}");
        println!("      ↳ score: {best_score}");

        Ok(())
    }
}

fn parse_grid(input: &str) -> Result<Grid<u32>, Solver08Error> {
    let width = input.lines().next().unwrap().len();
    let height = input.lines().count();

    let mut grid = Grid::new(width, height);

    for (y, line) in input.lines().enumerate() {
        for (x, col) in line.chars().enumerate() {
            if !col.is_ascii_digit() {
                return Err(Solver08Error::ParseGridNonAsciiDigit {
                    chr: col,
                    coord: (x, y).into(),
                });
            }

            *grid.cell_mut((x, y).into()).unwrap() = col as u32 - '0' as u32;
        }
    }

    Ok(grid)
}

fn iter_trees_in_dir(
    grid: &Grid<u32>,
    coord: GridCoord,
    (dx, dy): (isize, isize),
) -> impl Iterator<Item = &u32> {
    (1..).into_iter().map_while(move |i| {
        let coord = GridCoord {
            x: coord.x.checked_add_signed(dx * i)?,
            y: coord.y.checked_add_signed(dy * i)?,
        };
        grid.cell(coord)
    })
}

fn count_visible_trees_in_dir(
    grid: &Grid<u32>,
    coord: GridCoord,
    (dx, dy): (isize, isize),
) -> usize {
    let line = iter_trees_in_dir(grid, coord, (dx, dy));

    let mut total = 0;
    let our_height = *grid.cell(coord).unwrap();
    for height in line {
        total += 1;
        if height >= &our_height {
            break;
        }
    }
    total
}

fn scenic_score(grid: &Grid<u32>, coord: GridCoord) -> usize {
    let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    dirs.into_iter()
        .map(|(dx, dy)| count_visible_trees_in_dir(grid, coord, (dx, dy)))
        .product()
}

#[derive(thiserror::Error, Debug)]
enum Solver08Error {
    #[error(
        "Could not parse character `{chr}` at grid location {coord:?} as an ASCII numeric digit"
    )]
    ParseGridNonAsciiDigit { chr: char, coord: GridCoord },
}
