use std::{fmt, str::FromStr};

use owo_colors::Rgb;

#[derive(Debug)]
pub enum Jet {
    Left,
    Right,
}

impl Jet {
    /// Parse all characters into a vector of jets.
    pub fn parse_all(input: &str) -> Result<Vec<Self>, ParseJetError> {
        input
            .trim()
            .chars()
            .map(|c| c.to_string().parse())
            .collect()
    }
}

impl FromStr for Jet {
    type Err = ParseJetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<" => Ok(Self::Left),
            ">" => Ok(Self::Right),
            _ => Err(ParseJetError {
                found: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Could not parse string as Jet. Expected \"<\" or \">\", found {found}")]
pub struct ParseJetError {
    found: String,
}

impl fmt::Display for Jet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => write!(f, "←"),
            Self::Right => write!(f, "→"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Coord {
    /// Positive x is right
    pub x: usize,
    /// Positive y is up
    pub y: usize,
}

impl Coord {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub const fn x_as_u8(&self) -> u8 {
        super::pack_x_coord(self.x)
    }
}

impl From<(usize, usize)> for Coord {
    fn from((x, y): (usize, usize)) -> Self {
        Self::new(x, y)
    }
}

impl std::ops::Add for Coord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Add for &Coord {
    type Output = Coord;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

pub struct Piece<'a> {
    pub coords: &'a [Coord],
    pub color: Rgb,
}

pub const PIECES: [Piece<'_>; 5] = [
    // horizontal line (-)
    Piece {
        coords: &[
            Coord::new(0, 0),
            Coord::new(1, 0),
            Coord::new(2, 0),
            Coord::new(3, 0),
        ],
        color: Rgb(0, 240, 240),
    },
    // plus (+)
    Piece {
        coords: &[
            Coord::new(0, 1),
            Coord::new(1, 0),
            Coord::new(1, 1),
            Coord::new(1, 2),
            Coord::new(2, 1),
        ],
        color: Rgb(160, 0, 240),
    },
    // backwards L (⅃)
    Piece {
        coords: &[
            Coord::new(0, 0),
            Coord::new(1, 0),
            Coord::new(2, 0),
            Coord::new(2, 1),
            Coord::new(2, 2),
        ],
        color: Rgb(240, 160, 0),
    },
    // vertical line (|)
    Piece {
        coords: &[
            Coord::new(0, 0),
            Coord::new(0, 1),
            Coord::new(0, 2),
            Coord::new(0, 3),
        ],
        color: Rgb(0, 240, 0),
    },
    // square (▩)
    Piece {
        coords: &[
            Coord::new(0, 0),
            Coord::new(1, 0),
            Coord::new(0, 1),
            Coord::new(1, 1),
        ],
        color: Rgb(240, 240, 0),
    },
];
