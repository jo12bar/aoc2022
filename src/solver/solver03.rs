use std::{collections::HashSet, io::BufRead};

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver03;

impl ChallengeSolver for Solver03 {
    #[inline]
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        3
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut compartment_a = HashSet::new();
        let mut compartment_b = HashSet::new();

        let mut intersection_priority_sum = 0;

        for line in input.lines() {
            let line = line?;
            let line = line.trim();

            assert_eq!(
                line.len() % 2,
                0,
                "All bags must contain an even number of items!"
            );

            let (a, b) = line.split_at(line.len() / 2);

            assert!(
                a.len() == b.len(),
                "Each compartment must have same number of items!"
            );

            for (char_a, char_b) in a.chars().zip(b.chars()) {
                compartment_a.insert(char_a);
                compartment_b.insert(char_b);
            }

            for item in compartment_a.intersection(&compartment_b) {
                intersection_priority_sum += item_priority(*item);
            }

            compartment_a.drain();
            compartment_b.drain();
        }

        println!("Interseciton item priority sum: {intersection_priority_sum}");

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut elf_one = HashSet::new();
        let mut elf_two = HashSet::new();
        let mut elf_three = HashSet::new();
        let mut intersection_priority_sum = 0;

        for (i, line) in input.lines().enumerate() {
            let line = line?;
            let line = line.trim();

            if i % 3 == 0 {
                // first elf
                for item in line.chars() {
                    elf_one.insert(item);
                }
            } else if i % 3 == 1 {
                // second elf
                for item in line.chars() {
                    elf_two.insert(item);
                }
            } else if i % 3 == 2 {
                // third elf
                for item in line.chars() {
                    elf_three.insert(item);
                }

                // The badge is the only item common between all three elves
                for common_item in elf_one
                    .iter()
                    .filter(|item| elf_two.contains(item))
                    .filter(|item| elf_three.contains(item))
                {
                    intersection_priority_sum += item_priority(*common_item);
                }

                // drain all three elf hashsets for the next group
                elf_one.drain();
                elf_two.drain();
                elf_three.drain();
            }
        }

        println!("Interseciton item priority sum: {intersection_priority_sum}");

        Ok(Box::new(()))
    }
}

fn item_priority(item: char) -> u32 {
    match item {
        item @ 'a'..='z' => item as u32 - 96,
        item @ 'A'..='Z' => item as u32 - 38,
        _ => u32::MAX,
    }
}

#[test]
fn test_item_priority() {
    assert_eq!(item_priority('a'), 1);
    assert_eq!(item_priority('t'), 20);
    assert_eq!(item_priority('z'), 26);

    assert_eq!(item_priority('A'), 27);
    assert_eq!(item_priority('D'), 30);
    assert_eq!(item_priority('Z'), 52);

    assert_eq!(item_priority(' '), u32::MAX);
    assert_eq!(item_priority('😅'), u32::MAX);
}
