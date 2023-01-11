mod parse;

use std::io::BufRead;

use color_eyre::eyre::Context;
use miette::GraphicalReportHandler;
use nom_supreme::{
    error::{BaseErrorKind, ErrorTree, GenericErrorTree},
    final_parser::final_parser,
};

use self::parse::{parse_all_monkeys, Monkey, Span};

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver11;

impl ChallengeSolver for Solver11 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        11
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        // Parse the monkeys
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;

        let input = Span::new(&input_buf);

        let monkeys_res: Result<_, ErrorTree<Span>> =
            final_parser(parse_all_monkeys::<ErrorTree<Span>>)(input);

        let monkeys = match monkeys_res {
            Ok(monkeys) => monkeys,
            Err(e) => {
                match e {
                    GenericErrorTree::Base { location, kind } => {
                        let offset = location.location_offset().into();
                        let err = BadInputError {
                            src: &input_buf,
                            bad_bit: miette::SourceSpan::new(offset, 0.into()),
                            kind,
                        };
                        let mut s = String::new();
                        GraphicalReportHandler::new()
                            .render_report(&mut s, &err)
                            .unwrap();
                        eprintln!("{s}");
                    }

                    GenericErrorTree::Stack { .. } => todo!("generic error tree stack"),
                    GenericErrorTree::Alt(_) => todo!("generic error tree alt"),
                }
                return Err(color_eyre::eyre::eyre!("Failed to parse input"));
            }
        };

        // Simulate the monkeys
        let mut monkeys = monkeys;
        for i in 0..20 {
            println!("\n============");
            println!("| ROUND {i:<2} |");
            println!("============");

            do_round(&mut monkeys, true, None);
            for monkey in &monkeys {
                println!("{monkey:?}");
            }
        }

        // Calculate the resultant monkey business
        let mut all_inspect_counts = monkeys
            .iter()
            .map(|m| m.items_inspected)
            .collect::<Vec<_>>();
        all_inspect_counts.sort_unstable_by_key(|&c| std::cmp::Reverse(c));

        let monkey_business = all_inspect_counts.into_iter().take(2).product::<u128>();
        println!("\nMonkey business: {monkey_business}");

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        // Parse the monkeys
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;

        let input = Span::new(&input_buf);

        let monkeys_res: Result<_, ErrorTree<Span>> =
            final_parser(parse_all_monkeys::<ErrorTree<Span>>)(input);

        let monkeys = match monkeys_res {
            Ok(monkeys) => monkeys,
            Err(e) => {
                match e {
                    GenericErrorTree::Base { location, kind } => {
                        let offset = location.location_offset().into();
                        let err = BadInputError {
                            src: &input_buf,
                            bad_bit: miette::SourceSpan::new(offset, 0.into()),
                            kind,
                        };
                        let mut s = String::new();
                        GraphicalReportHandler::new()
                            .render_report(&mut s, &err)
                            .unwrap();
                        eprintln!("{s}");
                    }

                    GenericErrorTree::Stack { .. } => todo!("generic error tree stack"),
                    GenericErrorTree::Alt(_) => todo!("generic error tree alt"),
                }
                return Err(color_eyre::eyre::eyre!("Failed to parse input"));
            }
        };

        // Simulate the monkeys
        let divisor_product = monkeys.iter().map(|m| m.divisor).product::<u128>();
        dbg!(divisor_product);

        let mut monkeys = monkeys;
        for i in 0..10_000 {
            if i % 100 == 0 {
                println!("Round {i}");
            }

            do_round(&mut monkeys, false, Some(divisor_product));
        }

        // Calculate the resultant monkey business
        let mut all_inspect_counts = monkeys
            .iter()
            .map(|m| m.items_inspected)
            .collect::<Vec<_>>();
        all_inspect_counts.sort_unstable_by_key(|&c| std::cmp::Reverse(c));

        let monkey_business = all_inspect_counts.into_iter().take(2).product::<u128>();
        println!("\nMonkey business: {monkey_business}");

        Ok(Box::new(()))
    }
}

fn do_round(monkeys: &mut [Monkey], div_by_three: bool, divisor_product: Option<u128>) {
    let num_monkeys = monkeys.len();

    for i in 0..num_monkeys {
        let old_monkey;

        {
            let monkey = &mut monkeys[i];
            old_monkey = monkey.clone();
            monkey.items_inspected += old_monkey.items.len() as u128;
        }

        for mut item in old_monkey.items.iter().copied() {
            if let Some(divisor_product) = divisor_product {
                item %= divisor_product;
            }

            item = old_monkey.operation.eval(item);

            if div_by_three {
                item /= 3;
            }

            if item % old_monkey.divisor == 0 {
                monkeys[old_monkey.receiver_if_true].items.push(item);
            } else {
                monkeys[old_monkey.receiver_if_false].items.push(item);
            }
        }
        monkeys[i].items.clear();
    }
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("Error parsing input")]
struct BadInputError<'a> {
    #[source_code]
    src: &'a str,

    #[label("{kind}")]
    bad_bit: miette::SourceSpan,

    kind: BaseErrorKind<&'static str, Box<dyn std::error::Error + Send + Sync>>,
}
