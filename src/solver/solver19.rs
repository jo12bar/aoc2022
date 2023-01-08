use std::{
    fs::File,
    io::{BufReader, Read},
};

use color_eyre::eyre::Context;
use rayon::prelude::*;

use self::parse::{Blueprint, Resources};

mod parse;

#[derive(Debug, Default)]
pub struct Solver19;

impl super::ChallengeSolver for Solver19 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        19
    }

    fn solve_a(&mut self, mut input: BufReader<File>) -> color_eyre::Result<()> {
        let start_time = std::time::Instant::now();

        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;
        let blueprints = parse::parse_input(&input_buf)
            .wrap_err("Could not parse input file as a list of blueprints")?;

        let cumulative_quality = part_a(&blueprints);
        println!("cumulative quality: {cumulative_quality}");

        println!("elapsed time: {:?}", start_time.elapsed());

        Ok(())
    }

    fn solve_b(&mut self, mut input: BufReader<File>) -> color_eyre::Result<()> {
        let start_time = std::time::Instant::now();

        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;
        let blueprints = parse::parse_input(&input_buf)
            .wrap_err("Could not parse input file as a list of blueprints")?;

        let geode_product = part_b(&blueprints);
        println!("geode product: {geode_product}");

        println!("elapsed time: {:?}", start_time.elapsed());

        Ok(())
    }
}

fn part_a(blueprints: &[Blueprint]) -> usize {
    blueprints
        .par_iter()
        .map(|blueprint| {
            let mut best = 0;
            geode_dfs(blueprint, State::new(24), &mut best);
            blueprint.id as usize * best as usize
        })
        .sum()
}

fn part_b(blueprints: &[Blueprint]) -> usize {
    blueprints
        .iter()
        .take(3)
        .map(|blueprint| {
            let mut best = 0;
            geode_dfs(blueprint, State::new(32), &mut best);
            best as usize
        })
        .product()
}

/// Conduct a depth-first search of the optimal geode production technique given a blueprint,
/// a starting state, and a prior "best" geode count.
///
/// `best` will be set to a new best geode count if a higher count is found.
fn geode_dfs(blueprint: &Blueprint, state: State, best: &mut u8) {
    *best = state.geodes_secured.max(*best);

    for state in state.future_states(blueprint) {
        if state.possible_geodes(blueprint) > *best {
            geode_dfs(blueprint, state, best);
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    minutes_remaining: u8,
    geodes_secured: u8,
    resources: Resources,
    resources_rate: Resources,
}

impl State {
    fn new(minutes_remaining: u8) -> Self {
        Self {
            minutes_remaining,
            geodes_secured: 0,
            resources: Resources::default(),
            resources_rate: Resources::ONE_ORE,
        }
    }

    /// Try to build a robot at some point in the future.
    ///
    /// Returns `Some(State)` if a state was found at some point in the future
    /// where building the robot is viable, including the new count of minutes
    /// remaining, the new resources rate, and the new resources count.
    fn choose_robot(&self, cost: Resources, robot: Resources) -> Option<Self> {
        (1..self.minutes_remaining).rev().zip(0..).find_map(
            |(minutes_remaining, minutes_passed)| {
                // Figure out how many resources have been produced at this point in the future
                let resources = self.resources + self.resources_rate * minutes_passed;

                // If we have enough resources to build the requested robot, then
                // return the new state
                resources.checked_sub(cost).map(|remaining_resources| Self {
                    resources: remaining_resources + self.resources_rate,
                    resources_rate: self.resources_rate + robot,

                    minutes_remaining,
                    geodes_secured: self.geodes_secured,
                })
            },
        )
    }

    /// Return an iterator over the next possible States if any robots are
    /// able to be built in the future given the current State.
    fn future_states(self, blueprint: &Blueprint) -> impl Iterator<Item = Self> + '_ {
        let max_higher_tier_ore_cost = blueprint
            .clay_robot_cost
            .ore
            .max(blueprint.obsidian_robot_cost.ore)
            .max(blueprint.geode_robot_cost.ore);

        // Figure out which robots are "viable" to be built, always with a preference
        // to building higher-tier robots (up to geode robots).
        let ore_robot_viable = self.resources_rate.ore < max_higher_tier_ore_cost;
        let clay_robot_viable = self.resources_rate.clay < blueprint.obsidian_robot_cost.clay;
        let obsidian_robot_viable = self.resources_rate.obsidian
            < blueprint.geode_robot_cost.obsidian
            && self.resources_rate.clay > 0;
        let geode_robot_viable = self.resources_rate.obsidian > 0;

        [
            ore_robot_viable
                .then(|| self.choose_robot(blueprint.ore_robot_cost, Resources::ONE_ORE)),
            clay_robot_viable
                .then(|| self.choose_robot(blueprint.clay_robot_cost, Resources::ONE_CLAY)),
            obsidian_robot_viable
                .then(|| self.choose_robot(blueprint.obsidian_robot_cost, Resources::ONE_OBSIDIAN)),
            geode_robot_viable.then(|| {
                self.choose_robot(blueprint.geode_robot_cost, Default::default())
                    .map(|state| Self {
                        geodes_secured: state.geodes_secured + state.minutes_remaining,
                        ..state
                    })
            }),
        ]
        .into_iter()
        .flatten()
        .flatten()
    }

    /// Determine how many geodes can be produced if *only* geode robots are
    /// produced until time is up.
    fn possible_geodes(&self, blueprint: &Blueprint) -> u8 {
        let geode_robot_cost = blueprint.geode_robot_cost.obsidian;
        let (_, _, geodes) = (0..self.minutes_remaining).rev().fold(
            (
                self.resources.obsidian,
                self.resources_rate.obsidian,
                self.geodes_secured,
            ),
            |(obsidian, rate, geodes), minutes_remaining| {
                if obsidian >= geode_robot_cost {
                    (
                        obsidian + rate - geode_robot_cost,
                        rate,
                        geodes.saturating_add(minutes_remaining),
                    )
                } else {
                    (obsidian + rate, rate + 1, geodes)
                }
            },
        );
        geodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_INPUT: &str = "Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.\n\
                                Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.";

    #[test]
    fn test_a() -> color_eyre::Result<()> {
        assert_eq!(part_a(&parse::parse_input(SAMPLE_INPUT)?), 33);

        Ok(())
    }

    #[test]
    fn test_b() -> color_eyre::Result<()> {
        assert_eq!(part_b(&parse::parse_input(SAMPLE_INPUT)?), 56 * 62);

        Ok(())
    }
}
