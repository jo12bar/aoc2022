use std::fmt;

use miette::GraphicalReportHandler;
use nom::{
    character::complete::{self as nom_cc, multispace0, multispace1, space0},
    combinator::map,
    error::ParseError,
    sequence::{preceded, separated_pair, tuple},
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

/// Parse the challenge input into a vector of [`Record`]s.
///
/// Any parsing errors will be printed out to `stderr` with fancy formatting.
pub fn parse_input(input: &str) -> Result<Vec<Record>, ParseInputError> {
    let input_span = Span::new(input);

    let records_res: Result<_, ErrorTree<Span>> =
        final_parser(Record::parse_all::<ErrorTree<Span>>)(input_span);

    match records_res {
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

#[derive(Debug)]
pub struct Record {
    /// A sensor
    pub sensor: Point,
    /// The closest beacon to that sensor
    pub beacon: Point,
}

impl Record {
    /// Parse multiple newline-seperated records into a vector.
    pub fn parse_all<'a, E: ParseError<Span<'a>> + TagError<Span<'a>, &'static str>>(
        i: Span<'a>,
    ) -> IResult<Span<'a>, Vec<Self>, E> {
        collect_separated_terminated(
            Self::parse,
            multispace1,
            tuple((multispace0, Self::parse.peek().not())),
        )
        .parse(i)
    }

    /// Parses a record, including sensor location and beacon location.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let (_, record) = Point::parse(Span::new(
    ///     "Sensor at x=272, y=1998931: closest beacon is at x=10411, y=2000000"
    /// )).unwrap();
    ///
    /// assert_eq!(record.sensor, Point { x: 272, y: 1998931 });
    /// assert_eq!(record.beacon, Point { x: 10411, y: 2000000 });
    /// ```
    pub fn parse<'a, E: ParseError<Span<'a>> + TagError<Span<'a>, &'static str>>(
        i: Span<'a>,
    ) -> IResult<Span<'a>, Self, E> {
        map(
            separated_pair(
                preceded(tag("Sensor at "), Point::parse),
                tag(": closest beacon is at "),
                Point::parse,
            ),
            |(sensor, beacon)| Self { sensor, beacon },
        )(i)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    /// Parses a point.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let (_, point) = Point::parse(Span::new("x=-3, y=42")).unwrap();
    /// assert_eq!(point, Point { x: -3, y: 42 });
    /// ```
    pub fn parse<'a, E: ParseError<Span<'a>> + TagError<Span<'a>, &'static str>>(
        i: Span<'a>,
    ) -> IResult<Span<'a>, Self, E> {
        map(
            separated_pair(
                preceded(tag("x="), nom_cc::i64),
                tuple((tag(","), space0)),
                preceded(tag("y="), nom_cc::i64),
            ),
            |(x, y)| Self { x, y },
        )(i)
    }

    /// Calculate the [Manhattan Distance][wiki] between two points.
    ///
    /// [wiki]: https://en.wikipedia.org/wiki/Taxicab_geometry
    pub fn manhattan_dist(self, other: Self) -> i64 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as i64
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<(i64, i64)> for Point {
    fn from(value: (i64, i64)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}
