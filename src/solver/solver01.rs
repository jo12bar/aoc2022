use std::io::BufRead;

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver01;

impl ChallengeSolver for Solver01 {
    #[inline]
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        1
    }

    fn solve_a(&mut self, input: std::io::BufReader<std::fs::File>) -> color_eyre::Result<()> {
        let mut max = 0_u64;
        let mut current = 0_u64;

        for line in input.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() {
                // we've stopped processing the current elf. Update the max calorie count.
                max = max.max(current);
                current = 0;
            } else {
                // update the current elf's calorie count.
                current += line.parse::<u64>()?;
            }
        }

        println!("Max calorie count: {max}");

        Ok(())
    }

    fn solve_b(&mut self, input: std::io::BufReader<std::fs::File>) -> color_eyre::Result<()> {
        let mut top_three = [0_u64; 3];
        let mut current = 0_u64;

        for line in input.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() {
                // we've stopped processing the current elf. Update the top three calorie counts.
                if current > top_three[0] {
                    top_three[2] = top_three[1];
                    top_three[1] = top_three[0];
                    top_three[0] = current;
                } else if current > top_three[1] {
                    top_three[2] = top_three[1];
                    top_three[1] = current;
                } else if current > top_three[2] {
                    top_three[2] = current;
                }

                current = 0;
            } else {
                // update the current elf's calorie count.
                current += line.parse::<u64>()?;
            }
        }

        println!("Top three calorie counts: {top_three:?}");
        println!("Sum: {}", top_three.iter().sum::<u64>());

        Ok(())
    }
}
