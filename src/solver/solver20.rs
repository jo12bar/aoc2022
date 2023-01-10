use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use color_eyre::eyre::Context;

const PART_B_DECRYPTION_KEY: i64 = 811589153;

#[derive(Debug, Default)]
pub struct Solver20;

impl super::ChallengeSolver for Solver20 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        20
    }

    fn solve_a(&mut self, input: BufReader<File>) -> color_eyre::Result<()> {
        let nums = parse(input).wrap_err("Failed to parse challenge input")?;

        println!("grove coordinate sum = {}", solve(nums, 1, 1));

        Ok(())
    }

    fn solve_b(&mut self, input: BufReader<File>) -> color_eyre::Result<()> {
        let nums = parse(input).wrap_err("Failed to parse challenge input")?;

        println!(
            "grove coordinate sum = {}",
            solve(nums, PART_B_DECRYPTION_KEY, 10)
        );

        Ok(())
    }
}

fn parse(input: BufReader<File>) -> color_eyre::Result<Vec<i64>> {
    let mut nums = Vec::new();

    for line in input.lines() {
        let line = line.wrap_err("Could not read line from challenge input file")?;
        let num = line
            .parse::<i64>()
            .wrap_err_with(|| format!("Could not parse `{line}` as a 64-bit signed integer"))?;
        nums.push(num);
    }

    Ok(nums)
}

fn solve(numbers: Vec<i64>, decryption_key: i64, mixer_iterations: usize) -> i64 {
    let next_jump_size = (numbers.len() as f64 / 2.0).sqrt().floor() as usize;

    let numbers = numbers
        .into_iter()
        .map(|x| x * decryption_key)
        .collect::<Vec<_>>();

    let mut prev = (0..numbers.len() as u16).collect::<Vec<_>>();
    let mut next = prev.clone();

    prev.rotate_right(1);
    next.rotate_left(next_jump_size % numbers.len());

    for _ in 0..mixer_iterations {
        for (cur, &n) in numbers.iter().enumerate() {
            // remove cur from the list
            fix_pairs_backwards(prev[cur], next[cur], &mut prev, &mut next, cur as _);

            // find the node to insert cur after
            let amount_to_move = n.rem_euclid(numbers.len() as i64 - 1) as usize;
            let target = find_target(prev[cur], amount_to_move, next_jump_size, &prev, &next);

            // insert cur after the target
            prev[cur] = target;
            fix_pairs_backwards(
                cur as u16,
                next[target as usize],
                &mut prev,
                &mut next,
                target,
            );
        }
    }

    let zero_index = numbers
        .iter()
        .position(|&x| x == 0)
        .expect("challenge input does not contain an element with value 0");

    itertools::iterate(zero_index as u16, |&cur| {
        find_target(cur, 1000, next_jump_size, &prev, &next)
    })
    .skip(1)
    .take(3)
    .map(|i| numbers[i as usize])
    .sum()
}

fn fix_pairs_backwards(left: u16, right: u16, prev: &mut [u16], next: &mut [u16], stop: u16) {
    let (far_prev, immediate_next) = itertools::iterate(left, |&i| prev[i as usize])
        .zip(itertools::iterate(right, |&i| prev[i as usize]))
        .inspect(|&(before, after)| {
            next[before as usize] = after;
        })
        .find(|&(_, after)| prev[after as usize] == stop)
        .unwrap();
    prev[immediate_next as usize] = left;
    next[prev[far_prev as usize] as usize] = left;
}

fn find_target(
    from: u16,
    amount_to_move: usize,
    next_jump_size: usize,
    prev: &[u16],
    next: &[u16],
) -> u16 {
    let overshot_target = itertools::iterate(from, |&cur| next[cur as usize])
        .nth((next_jump_size + amount_to_move) / next_jump_size)
        .unwrap();
    itertools::iterate(overshot_target, |&cur| prev[cur as usize])
        .nth(next_jump_size - amount_to_move % next_jump_size)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> Vec<i64> {
        vec![1, 2, -3, 3, -2, 0, 4]
    }

    const SAMPLE_SOL_A: i64 = 3;
    const SAMPLE_SOL_B: i64 = 1623178306;

    #[test]
    fn test_a() -> color_eyre::Result<()> {
        let nums = sample_input();

        assert_eq!(solve(nums, 1, 1), SAMPLE_SOL_A);

        Ok(())
    }

    #[test]
    fn test_b() -> color_eyre::Result<()> {
        let nums = sample_input();

        assert_eq!(solve(nums, PART_B_DECRYPTION_KEY, 10), SAMPLE_SOL_B);

        Ok(())
    }
}
