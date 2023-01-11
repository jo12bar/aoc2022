mod parse;

use std::{collections::HashSet, io::BufRead, ops::RangeInclusive};

use color_eyre::eyre::Context;
use itertools::Itertools;

use self::parse::{Point, Record};

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver15;

impl ChallengeSolver for Solver15 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        15
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;

        let map = Map::parse(&input_buf)?;
        map.dump();

        let y = 2_000_000;
        dbg!(map.num_impossible_beacon_positions(y));

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;

        let map = Map::parse(&input_buf)?;
        map.dump();

        let y = 2_000_000;
        dbg!(map.num_impossible_beacon_positions(y));

        let range = 0..=4_000_000;
        let bp = map.beacon_position(&range, &range).unwrap();
        dbg!(bp);

        println!("tuning frequency = {}", bp.x * 4_000_000 + bp.y);

        Ok(Box::new(()))
    }
}

struct Map {
    records: Vec<Record>,
}

impl Map {
    fn parse(input: &str) -> Result<Self, MapError> {
        Ok(Self {
            records: parse::parse_input(input)?,
        })
    }

    /// Print all records to `stdout` using their [`std::fmt::Debug`] implementations.
    fn dump(&self) {
        for record in &self.records {
            println!("{record:?}");
        }
    }

    /// Returns a sorted iterator through all coverage ranges with a particular y-coordinate.
    fn ranges(&self, y: i64) -> impl Iterator<Item = RangeInclusive<i64>> {
        let mut ranges = Vec::new();
        for rec in &self.records {
            let radius = rec.sensor.manhattan_dist(rec.beacon);
            let y_dist = (y - rec.sensor.y).abs();

            if y_dist > radius {
                // coverage area doesn't touch line at `y`
                continue;
            }

            let d = radius - y_dist;
            let middle = rec.sensor.x;
            let start = middle - d;
            let end = middle + d;
            let range = start..=end;
            ranges.push(range);
        }
        ranges.sort_unstable_by_key(|r| *r.start());

        ranges.into_iter().coalesce(|a, b| {
            if b.start() - 1 <= *a.end() {
                if b.end() > a.end() {
                    Ok(*a.start()..=*b.end())
                } else {
                    Ok(a)
                }
            } else {
                Err((a, b))
            }
        })
    }

    /// Returns a sorted iterator through all coverage ranges with a particular y-coordinate,
    /// clamped to a particular range of x-coordinates.
    fn ranges_clamped(
        &self,
        y: i64,
        x_range: RangeInclusive<i64>,
    ) -> impl Iterator<Item = RangeInclusive<i64>> {
        self.ranges(y).filter_map(move |r| {
            // Make sure that `r` fits into `x_range`
            let r = *r.start().max(x_range.start())..=*r.end().min(x_range.end());
            if r.start() > r.end() {
                None
            } else {
                Some(r)
            }
        })
    }

    /// Return the number of impossible beacon positions with a particular y-coordinate.
    fn num_impossible_beacon_positions(&self, y: i64) -> usize {
        let beacon_x_coords = self
            .records
            .iter()
            .filter(|rec| rec.beacon.y == y)
            .map(|rec| rec.beacon.x)
            .collect::<HashSet<_>>();

        self.ranges(y)
            .map(|r| {
                let range_size = (r.end() - r.start() + 1) as usize;
                let num_beacons_in_range = beacon_x_coords.iter().filter(|x| r.contains(x)).count();
                range_size - num_beacons_in_range
            })
            .sum::<usize>()
    }

    // Return the position of a missing beacon, where its coordinates (x, y) are within
    // some range.
    fn beacon_position(
        &self,
        x_range: &RangeInclusive<i64>,
        y_range: &RangeInclusive<i64>,
    ) -> Option<Point> {
        y_range.clone().find_map(|y| {
            self.ranges_clamped(y, x_range.clone())
                .nth(1)
                .map(|r| Point {
                    x: r.start() - 1,
                    y,
                })
        })
    }
}

#[derive(thiserror::Error, Debug)]
enum MapError {
    #[error("Error parsing challenge input while building map")]
    ParseError {
        #[from]
        source: parse::ParseInputError,
    },
}
