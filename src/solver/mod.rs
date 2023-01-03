use std::{collections::HashMap, fmt, fs, io};

use crate::challenge::{ChallengeNumber, Subchallenge};

mod solver01;
mod solver02;
mod solver03;
mod solver04;
mod solver05;
mod solver06;
mod solver07;
mod solver08;
mod solver09;
mod solver10;
mod solver11;
mod solver12;
mod solver13;
mod solver14;
mod solver15;
mod solver16;

/// A solver for a single challenge.
///
/// Must be able to handle solving both subchallenges.
trait ChallengeSolver: fmt::Debug {
    /// The challenge number that this solver is written for.
    fn challenge_number(&self) -> ChallengeNumber;

    /// Solve subchallenge A.
    fn solve_a(&mut self, input: io::BufReader<fs::File>) -> color_eyre::Result<()>;

    /// Solve subchallenge B.
    fn solve_b(&mut self, input: io::BufReader<fs::File>) -> color_eyre::Result<()>;
}

type DynamicChallengeSolver = Box<dyn ChallengeSolver>;

pub struct Solver {
    challenge_solvers: HashMap<ChallengeNumber, DynamicChallengeSolver>,
}

impl Solver {
    pub fn new() -> Self {
        macro_rules! build_solver_list {
            [$($solver_ty:ty),* $(,)?] => {
                vec![
                    $(
                        Box::<$solver_ty>::default(),
                    )*
                ]
            };
        }

        let solvers: Vec<DynamicChallengeSolver> = build_solver_list![
            solver01::Solver01,
            solver02::Solver02,
            solver03::Solver03,
            solver04::Solver04,
            solver05::Solver05,
            solver06::Solver06,
            solver07::Solver07,
            solver08::Solver08,
            solver09::Solver09,
            solver10::Solver10,
            solver11::Solver11,
            solver12::Solver12,
            solver13::Solver13,
            solver14::Solver14,
            solver15::Solver15,
            solver16::Solver16,
        ];

        let mut challenge_solvers = HashMap::new();

        for solver in solvers {
            let challenge_number = solver.challenge_number();
            let previous = challenge_solvers.insert(challenge_number, solver);

            assert!(
                previous.is_none(),
                "Tried to load a duplicate solver for challenge {}",
                challenge_number,
            );
        }

        Self { challenge_solvers }
    }

    pub fn solve(
        &mut self,
        challenge: ChallengeNumber,
        subchallenge: Subchallenge,
        input: io::BufReader<fs::File>,
    ) -> Result<(), SolveError> {
        if let Some(solver) = self.challenge_solvers.get_mut(&challenge) {
            match subchallenge {
                Subchallenge::A => Ok(solver.solve_a(input)?),
                Subchallenge::B => Ok(solver.solve_b(input)?),
            }
        } else {
            Err(SolveError::NoSolverLoaded(challenge))
        }
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    #[error("No solver loaded for challenge {0}.")]
    NoSolverLoaded(ChallengeNumber),

    #[error(transparent)]
    SolverExecutionError(#[from] color_eyre::Report),
}
