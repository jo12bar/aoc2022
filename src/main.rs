use std::path::PathBuf;

use color_eyre::{eyre::Context, Help};

mod challenge;
mod grid;
mod solver;

/// Help text to display when we receive `-h` or `--help` on the command line.
const HELP: &str = "\
aoc2022

Solves Advent of Code 2022 challenges in questionably-valid ways.

USAGE:
  aoc2022 [OPTIONS] CHALLENGE_NUMBER SUBCHALLENGE

FLAGS:
  -h, --help                 Prints this help message and exit.

OPTIONS:
  --input INPUT_FILE_PATH    Use a specific file as the puzzle input. If this
                             flag is not provided, then by default aoc2022 will
                             look for and use a file named
                             <CHALLENGE_NUMBER><SUBCHALLENGE>.txt in the
                             `./input/` directory (e.g. ./input/1b.txt or
                             ./input/01A.txt or ./input/25a.txt or so on).

ARGS:
  <CHALLENGE_NUMBER>         The numeric challenge number to solve. May be
                             zero-padded - for example, passing `0022` or just
                             `22` will both execute the solver for challenge
                             number 22.

  <SUBCHALLENGE>             The subchallenge to execute. Must be `a`, `b`,
                             `A`, or `B`.

EXAMPLES:
  aoc2022 --help             Print this help message and exit.

  aoc2022 1 b                Execute the solver for challenge 1, subchallenge b,
                             using the input file `./input/01a.txt`.

  aoc2022 05 A --input custom.txt
                             Execute the solver for challenge 5, subchallenge b,
                             using the input file `./custom.txt`.
";

/// CLI app arguments.
#[derive(Debug)]
struct AppArgs {
    challenge: challenge::ChallengeNumber,
    subchallenge: challenge::Subchallenge,
    input_file: Option<PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}\nUSAGE: aoc2022 [OPTIONS] CHALLENGE_NUMBER SUBCHALLENGE");
            std::process::exit(1);
        }
    };

    let input_file_buf =
        challenge::get_challenge_input(args.challenge, args.subchallenge, &args.input_file)
            .wrap_err_with(|| {
                format!(
                    "Could not find input file for challenge {}, subchallenge {}",
                    args.challenge, args.subchallenge
                )
            });

    let input_file_buf = if args.input_file.is_some() {
        input_file_buf?
    } else {
        input_file_buf.with_suggestion(|| format!(
            "Make sure that the file `./input/{}{}.txt` exists, is readable, and contains valid UTF-8 data!",
            args.challenge,
            args.subchallenge
        ))?
    };

    let mut solver = solver::Solver::new();
    solver
        .solve(args.challenge, args.subchallenge, input_file_buf)
        .wrap_err_with(|| {
            format!(
                "Error while solving challenge {}, subchallenge {}",
                args.challenge, args.subchallenge,
            )
        })?;

    Ok(())
}

/// Parse CLI arguments.
fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = AppArgs {
        challenge: pargs.free_from_str()?,
        subchallenge: pargs.free_from_str()?,
        input_file: pargs.opt_value_from_os_str("--input", parse_path_arg)?,
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {remaining:?}");
    }

    Ok(args)
}

/// Parse an [`OsStr`][std::ffi::OsStr] into a [`PathBuf`].
///
/// Will never actually fail. Returns a `Result` purely for compatibility with
/// [`pico_args::Arguments::value_from_os_str`].
fn parse_path_arg(s: &std::ffi::OsStr) -> Result<PathBuf, &'static str> {
    Ok(s.into())
}
