use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver01;

impl ChallengeSolver for Solver01 {
    #[inline]
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        1
    }

    fn solve_a(&mut self, _input: std::io::BufReader<std::fs::File>) -> color_eyre::Result<()> {
        Err(color_eyre::eyre::eyre!("foo"))
    }

    fn solve_b(&mut self, _input: std::io::BufReader<std::fs::File>) -> color_eyre::Result<()> {
        todo!()
    }
}
