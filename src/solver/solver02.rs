use std::{io::BufRead, str::FromStr};

use color_eyre::eyre::Context;

use super::ChallengeSolver;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum RoundOutcome {
    Loss,
    Draw,
    Win,
}

impl RoundOutcome {
    fn score(&self) -> u32 {
        match self {
            Self::Loss => 0,
            Self::Draw => 3,
            Self::Win => 6,
        }
    }
}

impl FromStr for RoundOutcome {
    type Err = Solver02Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X" => Ok(Self::Loss),
            "Y" => Ok(Self::Draw),
            "Z" => Ok(Self::Win),
            something_else => Err(Solver02Error::RoundOutcomeParse(something_else.to_string())),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum RoShamBo {
    Rock,
    Paper,
    Scissors,
}

impl RoShamBo {
    fn score(&self) -> u32 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }

    fn play_against(&self, opponent: &Self) -> RoundOutcome {
        match (self, opponent) {
            (RoShamBo::Rock, RoShamBo::Rock) => RoundOutcome::Draw,
            (RoShamBo::Rock, RoShamBo::Paper) => RoundOutcome::Loss,
            (RoShamBo::Rock, RoShamBo::Scissors) => RoundOutcome::Win,
            (RoShamBo::Paper, RoShamBo::Rock) => RoundOutcome::Win,
            (RoShamBo::Paper, RoShamBo::Paper) => RoundOutcome::Draw,
            (RoShamBo::Paper, RoShamBo::Scissors) => RoundOutcome::Loss,
            (RoShamBo::Scissors, RoShamBo::Rock) => RoundOutcome::Loss,
            (RoShamBo::Scissors, RoShamBo::Paper) => RoundOutcome::Win,
            (RoShamBo::Scissors, RoShamBo::Scissors) => RoundOutcome::Draw,
        }
    }

    fn get_desired_play(opponent: &Self, desired_outcome: &RoundOutcome) -> Self {
        match (opponent, desired_outcome) {
            (RoShamBo::Rock, RoundOutcome::Loss) => Self::Scissors,
            (RoShamBo::Rock, RoundOutcome::Draw) => Self::Rock,
            (RoShamBo::Rock, RoundOutcome::Win) => Self::Paper,
            (RoShamBo::Paper, RoundOutcome::Loss) => Self::Rock,
            (RoShamBo::Paper, RoundOutcome::Draw) => Self::Paper,
            (RoShamBo::Paper, RoundOutcome::Win) => Self::Scissors,
            (RoShamBo::Scissors, RoundOutcome::Loss) => Self::Paper,
            (RoShamBo::Scissors, RoundOutcome::Draw) => Self::Scissors,
            (RoShamBo::Scissors, RoundOutcome::Win) => Self::Rock,
        }
    }
}

impl FromStr for RoShamBo {
    type Err = Solver02Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" | "X" => Ok(Self::Rock),
            "B" | "Y" => Ok(Self::Paper),
            "C" | "Z" => Ok(Self::Scissors),
            something_else => Err(Solver02Error::RoShamBoParse(something_else.to_string())),
        }
    }
}

#[derive(Debug, Default)]
pub struct Solver02;

impl ChallengeSolver for Solver02 {
    #[inline]
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        2
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut player_score = 0;

        for line in input.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            let mut cols = line.split(' ');

            let opponent = cols
                .next()
                .ok_or(Solver02Error::OpponentMoveGet)?
                .parse::<RoShamBo>()
                .wrap_err("Could not parse opponent's move.")?;

            let player = cols
                .next()
                .ok_or(Solver02Error::PlayerMoveGet)?
                .parse::<RoShamBo>()
                .wrap_err("Could not parse opponent's move.")?;

            let result = player.play_against(&opponent);

            player_score += player.score() + result.score();
        }

        println!("Total player score: {player_score}");

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut player_score = 0;

        for line in input.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            let mut cols = line.split(' ');

            let opponent = cols
                .next()
                .ok_or(Solver02Error::OpponentMoveGet)?
                .parse::<RoShamBo>()
                .wrap_err("Could not parse opponent's move.")?;

            let desired_outcome = cols
                .next()
                .ok_or(Solver02Error::DesiredRoundOutcomeGet)?
                .parse::<RoundOutcome>()
                .wrap_err("Could not parse desired round outcome.")?;

            let player = RoShamBo::get_desired_play(&opponent, &desired_outcome);

            let result = player.play_against(&opponent);

            assert_eq!(desired_outcome, result);

            player_score += player.score() + result.score();
        }

        println!("Total player score: {player_score}");

        Ok(Box::new(()))
    }
}

#[derive(Debug, thiserror::Error)]
enum Solver02Error {
    #[error("Could not parse `{0}` as a valid round outcome.")]
    RoundOutcomeParse(String),

    #[error("Could not parse `{0}` as a valid code for Rock, Paper, or Scissors.")]
    RoShamBoParse(String),

    #[error("Unable to get opponent's move.")]
    OpponentMoveGet,

    #[error("Unable to get player's move.")]
    PlayerMoveGet,

    #[error("Unable to get desired round outcome.")]
    DesiredRoundOutcomeGet,
}
