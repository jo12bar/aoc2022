use std::{collections::VecDeque, io::BufRead};

use itertools::Itertools;

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver06;

impl ChallengeSolver for Solver06 {
    #[inline]
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        6
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut buf = String::new();
        input.read_line(&mut buf).unwrap();

        let mut tokens = VecDeque::with_capacity(4);
        let mut processed_count = 0;
        let mut marker = None;

        for token in buf.chars() {
            if token == '\n' {
                break;
            }

            if tokens.len() == 4 {
                tokens.pop_front();
            }

            tokens.push_back(token);
            processed_count += 1;

            //println!("{tokens:?}");

            if tokens.iter().unique().count() == 4 {
                marker = Some(tokens.iter().join(""));
                break;
            }
        }

        if let Some(marker) = marker {
            println!("\nFound marker `{marker}` after processing {processed_count} characters");
        }

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut buf = String::new();
        input.read_line(&mut buf).unwrap();

        let mut tokens = VecDeque::with_capacity(4);
        let mut processed_count = 0;
        let mut marker = None;

        for token in buf.chars() {
            if token == '\n' {
                break;
            }

            if tokens.len() == 14 {
                tokens.pop_front();
            }

            tokens.push_back(token);
            processed_count += 1;

            //println!("{tokens:?}");

            if tokens.iter().unique().count() == 14 {
                marker = Some(tokens.iter().join(""));
                break;
            }
        }

        if let Some(marker) = marker {
            println!("\nFound marker `{marker}` after processing {processed_count} characters");
        }

        Ok(Box::new(()))
    }
}
