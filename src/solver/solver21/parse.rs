use miette::GraphicalReportHandler;
use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete as nom_cc,
    combinator::map,
    error::ParseError,
    sequence::{separated_pair, terminated, tuple},
    IResult, Parser,
};
use nom_locate::LocatedSpan;
use nom_supreme::{
    context::ContextError,
    error::{BaseErrorKind, ErrorTree, GenericErrorTree},
    final_parser::final_parser,
    multi::collect_separated_terminated,
    tag::{complete::tag, TagError},
    ParserExt,
};

use super::{Monkey, MonkeyRef, Op};

pub type Span<'a> = LocatedSpan<&'a str>;

/// Parse the challenge input into a vector of [`Blueprint`]s.
///
/// Any parsing errors will be printed out to `stderr` with fancy formatting.
pub(super) fn parse_input(input: &str) -> Result<Vec<Monkey>, ParseInputError> {
    let input_span = Span::new(input);

    let valves_res: Result<_, ErrorTree<Span>> =
        final_parser(parse_all_monkeys::<ErrorTree<Span>>)(input_span);

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

fn parse_monkey_name<'a, E>(i: Span<'a>) -> IResult<Span<'a>, String, E>
where
    E: ParseError<Span<'a>>,
{
    map(
        take_while1(|chr: char| chr.is_alphabetic()),
        |name: Span<'a>| name.to_string(),
    )(i)
}

fn parse_monkey_ref<'a, E>(i: Span<'a>) -> IResult<Span<'a>, MonkeyRef, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>, &'static str>,
{
    map(parse_monkey_name, MonkeyRef::Unresolved)
        .context("monkey reference")
        .parse(i)
}

fn parse_op<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Op, E>
where
    E: ParseError<Span<'a>>
        + TagError<Span<'a>, &'static str>
        + ContextError<Span<'a>, &'static str>,
{
    alt((
        map(nom_cc::i64, Op::Const).context("constant number"),
        map(
            separated_pair(parse_monkey_ref, tag(" + "), parse_monkey_ref),
            |(a, b)| Op::Add(a, b),
        )
        .context("addition operation"),
        map(
            separated_pair(parse_monkey_ref, tag(" - "), parse_monkey_ref),
            |(a, b)| Op::Sub(a, b),
        )
        .context("subtraction operation"),
        map(
            separated_pair(parse_monkey_ref, tag(" * "), parse_monkey_ref),
            |(a, b)| Op::Mul(a, b),
        )
        .context("multiplication operation"),
        map(
            separated_pair(parse_monkey_ref, tag(" / "), parse_monkey_ref),
            |(a, b)| Op::Div(a, b),
        )
        .context("division operation"),
    ))(i)
}

fn parse_monkey<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Monkey, E>
where
    E: ParseError<Span<'a>>
        + TagError<Span<'a>, &'static str>
        + ContextError<Span<'a>, &'static str>,
{
    let (i, name) = terminated(parse_monkey_name, tag(": "))
        .context("monkey name")
        .parse(i)?;
    let (i, op) = parse_op(i)?;

    Ok((i, Monkey { name, op }))
}

fn parse_all_monkeys<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Vec<Monkey>, E>
where
    E: ParseError<Span<'a>>
        + TagError<Span<'a>, &'static str>
        + ContextError<Span<'a>, &'static str>,
{
    collect_separated_terminated(
        parse_monkey,
        nom_cc::multispace1,
        tuple((nom_cc::multispace0, parse_monkey.peek().not())),
    )
    .parse(i)
}
