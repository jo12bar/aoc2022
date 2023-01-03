use std::fmt;

use miette::GraphicalReportHandler;
use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::{self as nom_cc, multispace0, multispace1},
    combinator::map,
    error::ParseError,
    multi::separated_list1,
    sequence::{preceded, tuple},
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

/// Parse the challenge input into a vector of [`Valve`]s.
///
/// Any parsing errors will be printed out to `stderr` with fancy formatting.
pub fn parse_input(input: &str) -> Result<Vec<Valve>, ParseInputError> {
    let input_span = Span::new(input);

    let valves_res: Result<_, ErrorTree<Span>> =
        final_parser(Valve::parse_all::<ErrorTree<Span>>)(input_span);

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

/// Valve names are always two characters: this is more compact than a `String`,
/// doesn't involves heap allocations, and gives us a `Copy` type, which is
/// really convenient.
///
/// The number of unique valve names are 26^2 = 676.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Name(pub [u8; 2]);

/// The maximum value that a [`Name`] can be converted to using [`Name::as_usize()`].
pub const MAX_NAME: usize = 26_usize.pow(2);

impl Name {
    fn parse<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>>,
    {
        map(take(2_usize), |slice: Span<'a>| {
            Self(slice.as_bytes().try_into().unwrap())
        })(i)
    }

    /// Returns this name as a `usize` between 0 and 26^2 (= 676).
    pub fn as_usize(self) -> usize {
        let [a, b] = self.0;

        debug_assert!(
            (b'A'..=b'Z').contains(&a),
            "`a` had a value outside the range {}..={}",
            b'A',
            b'Z'
        );
        debug_assert!(
            (b'A'..=b'Z').contains(&b),
            "`b` had a value outside the range {}..={}",
            b'A',
            b'Z'
        );

        (a - b'A') as usize * 26 + (b - b'A') as usize
    }

    /// Returns a name from a `usize` between 0 and 26^2 (= 676).
    ///
    /// In debug builds, if `index` >= [`MAX_NAME`], then the function will
    /// panic.
    pub fn from_usize(index: usize) -> Self {
        debug_assert!(
            index < MAX_NAME,
            "`index` must be less than {MAX_NAME}; found index == {index}"
        );

        let a = (index / 26) as u8 + b'A';
        let b = (index % 26) as u8 + b'A';
        Self([a, b])
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b] = self.0;
        write!(f, "{}{}", a as char, b as char)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct Valve {
    pub name: Name,
    pub flow: u64,
    pub links: Vec<Name>,
}

impl Valve {
    /// Parse multiple newline-seperated valves into a vector.
    fn parse_all<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Vec<Self>, E>
    where
        E: ParseError<Span<'a>> + TagError<Span<'a>, &'static str>,
    {
        collect_separated_terminated(
            Self::parse,
            multispace1,
            tuple((multispace0, Self::parse.peek().not())),
        )
        .parse(i)
    }

    /// Parse a valve.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let valve = Valve::parse(Span::new(
    ///     "Valve AA has flow rate=10; tunnels lead to valves DD, II, BB"
    /// )).unwrap().1;
    ///
    /// assert_eq!(format!("{}", valve.name), "AA".to_string());
    /// assert_eq!(valve.flow, 10);
    /// assert_eq!(
    ///     format!("{:?}", valve.links),
    ///     "[DD, II, BB]".to_string(),
    /// );
    /// ```
    fn parse<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + TagError<Span<'a>, &'static str>,
    {
        map(
            tuple((
                preceded(tag("Valve "), Name::parse),
                preceded(tag(" has flow rate="), nom_cc::u64),
                preceded(
                    alt((
                        tag("; tunnels lead to valves "),
                        tag("; tunnel leads to valve "),
                    )),
                    separated_list1(tag(", "), Name::parse),
                ),
            )),
            |(name, flow, links)| Self { name, flow, links },
        )(i)
    }
}
