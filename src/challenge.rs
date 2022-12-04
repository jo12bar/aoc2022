use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use regex::Regex;
use thiserror::Error;

pub type ChallengeNumber = u8;

#[derive(Copy, Clone, Debug)]
pub enum Subchallenge {
    A,
    B,
}

impl Subchallenge {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
        }
    }
}

impl fmt::Display for Subchallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl FromStr for Subchallenge {
    type Err = SubchallengeFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.len() != 1 {
            return Err(SubchallengeFromStrError(s.to_string()));
        }

        match s {
            "a" | "A" => Ok(Self::A),
            "b" | "B" => Ok(Self::B),
            something_else => Err(SubchallengeFromStrError(something_else.to_string())),
        }
    }
}

#[derive(Debug, Error)]
#[error(
    "Could not parse valid subchallenge designation from input. Expected one of `a`, `A`, `b`, or \
     `B`. Found `{0}`."
)]
pub struct SubchallengeFromStrError(String);

pub fn get_challenge_input(
    challenge: ChallengeNumber,
    subchallenge: Subchallenge,
    path_override: &Option<PathBuf>,
) -> Result<io::BufReader<fs::File>, GetChallengeInputError> {
    let path = if let Some(path) = path_override {
        path.clone()
    } else {
        find_default_challenge_input_file(challenge, subchallenge)?
    };

    let f = fs::File::open(path)?;

    Ok(io::BufReader::new(f))
}

fn find_default_challenge_input_file(
    challenge: ChallengeNumber,
    subchallenge: Subchallenge,
) -> Result<PathBuf, GetChallengeInputError> {
    let default_input_file_re =
        Regex::new(format!("(?i)^0*{challenge}{subchallenge}.txt$").as_str()).unwrap();

    let input_dir = Path::new("./input");

    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;

        if let Some(file_name) = entry.file_name().to_str() {
            if default_input_file_re.is_match(file_name) {
                return Ok(input_dir.join(file_name));
            }
        }
    }

    Err(GetChallengeInputError::DefaultInputFileLocationError {
        challenge,
        subchallenge,
    })
}

#[derive(Debug, Error)]
pub enum GetChallengeInputError {
    #[error(
        "Could not find default input file for challenge {challenge}, subchallenge {subchallenge}."
    )]
    DefaultInputFileLocationError {
        challenge: ChallengeNumber,
        subchallenge: Subchallenge,
    },

    #[error(transparent)]
    IoError(#[from] io::Error),
}
