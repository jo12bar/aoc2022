use std::{io::BufRead, ops::RangeInclusive};

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver04;

impl ChallengeSolver for Solver04 {
    #[inline]
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        4
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut containing_range_count = 0;

        for line in input.lines() {
            let line = line?;
            let line = line.trim();

            let (first_range_str, second_range_str) = line.split_once(',').unwrap();

            let (first_range_lower_bound, first_range_upper_bound) = first_range_str
                .split_once('-')
                .map(|(a, b)| (a.parse::<u32>().unwrap(), b.parse::<u32>().unwrap()))
                .unwrap();
            let first_range = first_range_lower_bound..=first_range_upper_bound;

            let (second_range_lower_bound, second_range_upper_bound) = second_range_str
                .split_once('-')
                .map(|(a, b)| (a.parse::<u32>().unwrap(), b.parse::<u32>().unwrap()))
                .unwrap();
            let second_range = second_range_lower_bound..=second_range_upper_bound;

            if range_contains_other(&first_range, &second_range)
                || range_contains_other(&second_range, &first_range)
            {
                println!("Found containing range pair: {first_range:?} and {second_range:?}");
                containing_range_count += 1;
            }
        }

        println!("Containing range count: {containing_range_count}");

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut overlapping_range_count = 0;

        for line in input.lines() {
            let line = line?;
            let line = line.trim();

            let (first_range_str, second_range_str) = line.split_once(',').unwrap();

            let (first_range_lower_bound, first_range_upper_bound) = first_range_str
                .split_once('-')
                .map(|(a, b)| (a.parse::<u32>().unwrap(), b.parse::<u32>().unwrap()))
                .unwrap();
            let first_range = first_range_lower_bound..=first_range_upper_bound;

            let (second_range_lower_bound, second_range_upper_bound) = second_range_str
                .split_once('-')
                .map(|(a, b)| (a.parse::<u32>().unwrap(), b.parse::<u32>().unwrap()))
                .unwrap();
            let second_range = second_range_lower_bound..=second_range_upper_bound;

            if ranges_overlap(&first_range, &second_range) {
                println!("Found overlapping range pair: {first_range:?} and {second_range:?}");
                overlapping_range_count += 1;
            }
        }

        println!("Overlapping range count: {overlapping_range_count}");

        Ok(Box::new(()))
    }
}

#[inline]
fn range_contains_other(range: &RangeInclusive<u32>, other: &RangeInclusive<u32>) -> bool {
    range.start() <= other.start() && other.end() <= range.end()
}

#[inline]
fn ranges_overlap(a: &RangeInclusive<u32>, b: &RangeInclusive<u32>) -> bool {
    a.start() <= b.end() && b.start() <= a.end()
}
