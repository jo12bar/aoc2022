use std::ops;

use miette::GraphicalReportHandler;
use nom::{
    character::complete as nom_cc,
    error::ParseError,
    sequence::{delimited, separated_pair, tuple},
    IResult, Parser,
};
use nom_locate::LocatedSpan;
use nom_supreme::{
    error::{BaseErrorKind, ErrorTree, GenericErrorTree},
    final_parser::final_parser,
    multi::collect_separated_terminated,
    tag::{complete::tag, TagError},
    ParserExt,
};

pub type Span<'a> = LocatedSpan<&'a str>;

/// Parse the challenge input into a vector of [`Blueprint`]s.
///
/// Any parsing errors will be printed out to `stderr` with fancy formatting.
pub fn parse_input(input: &str) -> Result<Vec<Blueprint>, ParseInputError> {
    let input_span = Span::new(input);

    let valves_res: Result<_, ErrorTree<Span>> =
        final_parser(Blueprint::parse_all::<ErrorTree<Span>>)(input_span);

    match valves_res {
        Ok(records) => Ok(records),

        Err(e) => match e {
            GenericErrorTree::Base { location, kind } => {
                let offset = location.location_offset().into();
                let err = BadInputError {
                    src: input.to_string(),
                    bad_bit: miette::SourceSpan::new(offset, 0.into()),
                    kind,
                };

                let mut s = String::new();
                GraphicalReportHandler::new()
                    .render_report(&mut s, &err)
                    .unwrap();
                eprintln!("{s}");

                Err(err.into())
            }

            GenericErrorTree::Stack { .. } => todo!("generic error tree stack"),
            GenericErrorTree::Alt(_) => todo!("generic error tree alt"),
        },
    }
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("Error parsing input")]
pub struct BadInputError {
    #[source_code]
    src: String,

    #[label("{kind}")]
    bad_bit: miette::SourceSpan,

    kind: BaseErrorKind<&'static str, Box<dyn std::error::Error + Send + Sync>>,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseInputError {
    #[error("Failed to parse input due to bad input")]
    BadInputError {
        #[from]
        source: BadInputError,
    },
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub id: u8,
    pub ore_robot_cost: Resources,
    pub clay_robot_cost: Resources,
    pub obsidian_robot_cost: Resources,
    pub geode_robot_cost: Resources,
}

impl Blueprint {
    fn parse_all<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Vec<Self>, E>
    where
        E: ParseError<Span<'a>> + TagError<Span<'a>, &'static str>,
    {
        collect_separated_terminated(
            Self::parse,
            nom_cc::multispace1,
            tuple((nom_cc::multispace0, Self::parse.peek().not())),
        )
        .parse(i)
    }

    fn parse<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + TagError<Span<'a>, &'static str>,
    {
        let (i, id) = delimited(tag("Blueprint "), nom_cc::u8, tag(": "))(i)?;

        let (i, ore_robot_cost) =
            delimited(tag("Each ore robot costs "), nom_cc::u8, tag(" ore. "))
                .map(|ore| Resources::ONE_ORE * ore)
                .parse(i)?;

        let (i, clay_robot_cost) =
            delimited(tag("Each clay robot costs "), nom_cc::u8, tag(" ore. "))
                .map(|ore| Resources::ONE_ORE * ore)
                .parse(i)?;

        let (i, obsidian_robot_cost) = delimited(
            tag("Each obsidian robot costs "),
            separated_pair(nom_cc::u8, tag(" ore and "), nom_cc::u8),
            tag(" clay. "),
        )
        .map(|(ore, clay)| Resources::ONE_ORE * ore + Resources::ONE_CLAY * clay)
        .parse(i)?;

        let (i, geode_robot_cost) = delimited(
            tag("Each geode robot costs "),
            separated_pair(nom_cc::u8, tag(" ore and "), nom_cc::u8),
            tag(" obsidian."),
        )
        .map(|(ore, obsidian)| Resources::ONE_ORE * ore + Resources::ONE_OBSIDIAN * obsidian)
        .parse(i)?;

        Ok((
            i,
            Self {
                id,
                ore_robot_cost,
                clay_robot_cost,
                obsidian_robot_cost,
                geode_robot_cost,
            },
        ))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Resources {
    pub ore: u8,
    pub clay: u8,
    pub obsidian: u8,
}

impl Resources {
    pub const ONE_ORE: Self = Self {
        ore: 1,
        clay: 0,
        obsidian: 0,
    };
    pub const ONE_CLAY: Self = Self {
        ore: 0,
        clay: 1,
        obsidian: 0,
    };
    pub const ONE_OBSIDIAN: Self = Self {
        ore: 0,
        clay: 0,
        obsidian: 1,
    };

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        Some(Self {
            ore: self.ore.checked_sub(rhs.ore)?,
            clay: self.clay.checked_sub(rhs.clay)?,
            obsidian: self.obsidian.checked_sub(rhs.obsidian)?,
        })
    }
}

impl ops::Add for Resources {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            ore: self.ore + rhs.ore,
            clay: self.clay + rhs.clay,
            obsidian: self.obsidian + rhs.obsidian,
        }
    }
}

impl ops::Mul<u8> for Resources {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self::Output {
        Self {
            ore: self.ore * rhs,
            clay: self.clay * rhs,
            obsidian: self.obsidian * rhs,
        }
    }
}
