use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{self as cc, newline, one_of, space0, space1},
    combinator::{map, value},
    error::ParseError,
    multi::separated_list1,
    sequence::{preceded, terminated, tuple},
    IResult,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

pub fn parse_all_monkeys<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> IResult<Span<'a>, Vec<Monkey>, E> {
    terminated(
        separated_list1(cc::multispace1, Monkey::parse),
        cc::multispace0,
    )(i)
}

#[derive(Debug, Clone)]
pub struct Monkey {
    pub id: usize,
    pub items_inspected: u128,
    pub items: Vec<u128>,
    pub operation: Operation,
    pub divisor: u128,
    pub receiver_if_true: usize,
    pub receiver_if_false: usize,
}

impl Monkey {
    /// Try to parse a monkey description.
    pub fn parse<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span<'a>, Self, E> {
        let (i, (_, _, id, _)) = tuple((
            space0,
            tag("Monkey "),
            map(cc::u64, |x| x as usize),
            tag(":\n"),
        ))(i)?;

        let (i, (_, _, _, items, _)) = tuple((
            space1,
            tag("Starting items:"),
            space0,
            separated_list1(tuple((tag(","), space0)), cc::u128),
            newline,
        ))(i)?;

        let (i, (_, _, _, operation, _)) =
            tuple((space1, tag("Operation:"), space0, Operation::parse, newline))(i)?;
        let (i, (_, _, _, divisor, _)) =
            tuple((space1, tag("Test: divisible by"), space0, cc::u128, newline))(i)?;

        let (i, (_, _, _, receiver_if_true, _)) = tuple((
            space1,
            tag("If true: throw to monkey"),
            space0,
            map(cc::u64, |x| x as usize),
            newline,
        ))(i)?;
        let (i, (_, _, _, receiver_if_false)) = tuple((
            space1,
            tag("If false: throw to monkey"),
            space0,
            map(cc::u64, |x| x as usize),
        ))(i)?;

        Ok((
            i,
            Self {
                id,
                items_inspected: 0,
                items,
                operation,
                divisor,
                receiver_if_true,
                receiver_if_false,
            },
        ))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Add(Term, Term),
    Mul(Term, Term),
}

impl Operation {
    /// Evaluate an operation given an old worry value.
    pub fn eval(self, old: u128) -> u128 {
        match self {
            Operation::Add(l, r) => l.eval(old) + r.eval(old),
            Operation::Mul(l, r) => l.eval(old) * r.eval(old),
        }
    }

    /// Try to parse an operation.
    pub fn parse<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span<'a>, Self, E> {
        let (i, (l, op, r)) = preceded(
            preceded(tag("new"), preceded(space0, preceded(tag("="), space0))),
            tuple((
                Term::parse,
                preceded(space0, one_of("*+")),
                preceded(space0, Term::parse),
            )),
        )(i)?;

        let op = match op {
            '*' => Operation::Mul(l, r),
            '+' => Operation::Add(l, r),
            _ => unreachable!(),
        };

        Ok((i, op))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Term {
    Old,
    Constant(u128),
}

impl Term {
    /// If `self` is a `Term::Old`, then `old` will be returned. Otherwise, if
    /// `self` is a `Term::Contant(c)`, then `c` will be returned.
    pub fn eval(self, old: u128) -> u128 {
        match self {
            Term::Old => old,
            Term::Constant(c) => c,
        }
    }

    /// Try to parse a term.
    pub fn parse<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span<'a>, Self, E> {
        alt((value(Self::Old, tag("old")), map(cc::u128, Self::Constant)))(i)
    }
}
