use std::io::BufRead;

use itertools::Itertools;

use super::ChallengeSolver;

type Crate = String;

#[derive(Debug, Default)]
pub struct Solver05;

impl ChallengeSolver for Solver05 {
    #[inline]
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        5
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        // We can assume that there will always be 9 stacks of crates.
        const EMPTY_STACK: Vec<Crate> = Vec::new();
        let mut stacks: [Vec<Crate>; 9] = [EMPTY_STACK; 9];
        let mut stacks_built = false;

        for line in input.lines() {
            clear_terminal();
            let line = line?;

            // First, build up the stacks...
            if !stacks_built {
                if line[1..2].chars().next().unwrap().is_numeric() {
                    // If the first non-whitespace character is a number, we've reached the stack labels.
                    stacks_built = true;

                    // At this point, the stacks are actually upside-down. Flip them!
                    for stack in stacks.iter_mut() {
                        let reversed = stack.iter().rev().cloned().collect::<Vec<_>>();
                        *stack = reversed;
                    }
                } else {
                    // Otherwise, just keep accumulating crates into stacks.

                    // Once the line is trimmed, crate labels only occur in columns 2, 6, 10, 14, 18,
                    // 22, 26, 30, and 34.
                    for (i, (_, chr)) in line
                        .char_indices()
                        .filter(|(i, _)| [1, 5, 9, 13, 17, 21, 25, 29, 33].contains(i))
                        .enumerate()
                    {
                        if !chr.is_whitespace() {
                            stacks[i].push(chr.to_string());
                        }
                    }
                    println!();
                }
            } else {
                // Once the stacks are built, start processing moves.
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // We display the stack *before* each move.
                print_stacks(&stacks);

                // Parse the move command.
                let mut move_count: usize = 0;
                let mut src: usize = 0;
                let mut dest: usize = 0;

                for (keyword, param) in line.split_whitespace().tuples() {
                    match keyword {
                        "move" => {
                            move_count = param.parse().unwrap();
                        }
                        "from" => {
                            src = param.parse().unwrap();
                        }
                        "to" => {
                            dest = param.parse().unwrap();
                        }
                        something_else => {
                            return Err(color_eyre::eyre::eyre!(
                                "Unknown keyword: {something_else}"
                            ))
                        }
                    }
                }

                // Execute the move command.
                for _ in 0..move_count {
                    if let Some(crte) = stacks[src - 1].pop() {
                        stacks[dest - 1].push(crte);
                    }
                }

                println!("Moving {move_count} crates from stack {src} to stack {dest}...");
            }
        }

        print_stacks(&stacks);

        let stack_tops = stacks
            .into_iter()
            .map(|stack| stack.last().unwrap().clone())
            .reduce(|acc, s| acc + &s)
            .unwrap();

        println!("\n\nStack tops: {stack_tops}");

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        // We can assume that there will always be 9 stacks of crates.
        const EMPTY_STACK: Vec<Crate> = Vec::new();
        let mut stacks: [Vec<Crate>; 9] = [EMPTY_STACK; 9];
        let mut stacks_built = false;

        for line in input.lines() {
            clear_terminal();
            let line = line?;

            // First, build up the stacks...
            if !stacks_built {
                if line[1..2].chars().next().unwrap().is_numeric() {
                    // If the first non-whitespace character is a number, we've reached the stack labels.
                    stacks_built = true;

                    // At this point, the stacks are actually upside-down. Flip them!
                    for stack in stacks.iter_mut() {
                        let reversed = stack.iter().rev().cloned().collect::<Vec<_>>();
                        *stack = reversed;
                    }
                } else {
                    // Otherwise, just keep accumulating crates into stacks.

                    // Once the line is trimmed, crate labels only occur in columns 2, 6, 10, 14, 18,
                    // 22, 26, 30, and 34.
                    for (i, (_, chr)) in line
                        .char_indices()
                        .filter(|(i, _)| [1, 5, 9, 13, 17, 21, 25, 29, 33].contains(i))
                        .enumerate()
                    {
                        if !chr.is_whitespace() {
                            stacks[i].push(chr.to_string());
                        }
                    }
                    println!();
                }
            } else {
                // Once the stacks are built, start processing moves.
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // We display the stack *before* each move.
                print_stacks(&stacks);

                // Parse the move command.
                let mut move_count: usize = 0;
                let mut src: usize = 0;
                let mut dest: usize = 0;

                for (keyword, param) in line.split_whitespace().tuples() {
                    match keyword {
                        "move" => {
                            move_count = param.parse().unwrap();
                        }
                        "from" => {
                            src = param.parse().unwrap();
                        }
                        "to" => {
                            dest = param.parse().unwrap();
                        }
                        something_else => {
                            return Err(color_eyre::eyre::eyre!(
                                "Unknown keyword: {something_else}"
                            ))
                        }
                    }
                }

                // Execute the move command.
                let mut buf = Vec::new();
                for _ in 0..move_count {
                    if let Some(crte) = stacks[src - 1].pop() {
                        buf.push(crte);
                    }
                }

                stacks[dest - 1].extend(buf.into_iter().rev());

                println!("Moving {move_count} crates from stack {src} to stack {dest}...");
            }
        }

        print_stacks(&stacks);

        let stack_tops = stacks
            .into_iter()
            .map(|stack| stack.last().unwrap().clone())
            .reduce(|acc, s| acc + &s)
            .unwrap();

        println!("\n\nStack tops: {stack_tops}");

        Ok(Box::new(()))
    }
}

fn clear_terminal() {
    print!("\x1B[2J");
}

fn print_stacks(stacks: &[Vec<Crate>; 9]) {
    print!("\x1B[1;1H");

    let mut grid = Vec::new();

    let tallest_stack = stacks
        .iter()
        .map(|stack| stack.len())
        .reduce(|acc, l| acc.max(l))
        .unwrap();

    for (i, stack) in stacks.iter().enumerate() {
        grid.push(vec![None; tallest_stack]);
        for (j, crte) in stack.iter().enumerate() {
            grid[i][tallest_stack - 1 - j] = Some(crte);
        }
    }

    let grid = transpose(grid);

    for line in grid.iter() {
        for crte in line.iter() {
            if let Some(crte) = crte {
                print!("[{crte}] ");
            } else {
                print!("    ");
            }
        }
        println!();
    }

    println!(" 1   2   3   4   5   6   7   8   9\n");
}

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    assert!(!v.is_empty());
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}
