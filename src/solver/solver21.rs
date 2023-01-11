mod parse;

use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

use color_eyre::eyre::Context;
use itertools::Itertools;

#[derive(Debug, Default)]
pub struct Solver21;

impl super::ChallengeSolver for Solver21 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        21
    }

    fn solve_a(&mut self, input: &mut dyn std::io::BufRead) -> super::ChallengeSolverResult {
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read challenge input to string")?;

        let mut monkeys = parse::parse_input(&input_buf)
            .wrap_err("Failed to parse challenge input as a list of monkeys")?;

        let (root_idx, _humn_idx) = resolve_monkeys(&mut monkeys);
        let root_idx = root_idx
            .ok_or_else(|| color_eyre::eyre::eyre!("Challenge input is missing a `root` monkey"))?;

        let root_res = monkeys[root_idx].get_value(&monkeys)?;

        dbg!(root_idx);
        dbg!(root_res);

        Ok(Box::new(root_res))
    }

    fn solve_b(&mut self, input: &mut dyn std::io::BufRead) -> super::ChallengeSolverResult {
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read challenge input to string")?;

        let mut monkeys = parse::parse_input(&input_buf)
            .wrap_err("Failed to parse challenge input as a list of monkeys")?;

        let (root_idx, humn_idx) = resolve_monkeys(&mut monkeys);
        let root_idx = root_idx
            .ok_or_else(|| color_eyre::eyre::eyre!("Challenge input is missing a `root` monkey"))?;
        let humn_idx = humn_idx
            .ok_or_else(|| color_eyre::eyre::eyre!("Challenge input is missing a `humn` monkey"))?;

        println!("root = {} ({})", root_idx, &monkeys[root_idx]);
        println!("humn = {} ({})", humn_idx, &monkeys[humn_idx]);

        let mut queue: VecDeque<(usize, i64)> = VecDeque::new(); // (index, expected value)

        if let Some((lhs_ref, rhs_ref)) = monkeys[root_idx].op.monkey_refs() {
            if let (Some(lhs_idx), Some(rhs_idx)) = (lhs_ref.resolved_idx(), rhs_ref.resolved_idx())
            {
                queue.push_back((rhs_idx, monkeys.get_value(lhs_idx)?));
                queue.push_back((lhs_idx, monkeys.get_value(rhs_idx)?));
            }
        }

        while let Some((i, expected)) = queue.pop_front() {
            if i == humn_idx {
                println!("expected = {expected}");
                return Ok(Box::new(expected));
            }

            if let Some((lhs_ref, rhs_ref)) = monkeys[i].op.monkey_refs() {
                if let (Some(lhs_idx), Some(rhs_idx)) =
                    (lhs_ref.resolved_idx(), rhs_ref.resolved_idx())
                {
                    queue.push_back((lhs_idx, monkeys.get_expected_lhs(i, expected)?));
                    queue.push_back((rhs_idx, monkeys.get_expected_rhs(i, expected)?));
                }
            }
        }

        eprintln!("Ran out of monkeys to search through!");
        Ok(Box::new(-1_i64))
    }
}

/// Resolve all references to other monkeys in each monkey's operation, and
/// return the index of the `root` monkey and the index of the human (`humn`) in
/// the passed-in slice.
fn resolve_monkeys(monkeys: &mut [Monkey]) -> (Option<usize>, Option<usize>) {
    let name_to_index: HashMap<String, usize> = monkeys
        .iter()
        .enumerate()
        .map(|(i, monkey)| (monkey.name.clone(), i))
        .collect();

    for monkey in monkeys.iter_mut() {
        if monkey.op.lhs_ref_unresolved() {
            let (lhs, _) = monkey.op.monkey_refs().unwrap();
            let lhs_name = lhs.unresolved_name().unwrap();
            let lhs_idx = name_to_index.get(lhs_name);

            if let Some(lhs_idx) = lhs_idx {
                monkey.op.set_lhs(MonkeyRef::Resolved(*lhs_idx));
            }
        }

        if monkey.op.rhs_ref_unresolved() {
            let (_, rhs) = monkey.op.monkey_refs().unwrap();
            let rhs_name = rhs.unresolved_name().unwrap();
            let rhs_idx = name_to_index.get(rhs_name);

            if let Some(rhs_idx) = rhs_idx {
                monkey.op.set_rhs(MonkeyRef::Resolved(*rhs_idx));
            }
        }
    }

    (
        name_to_index.get("root").copied(),
        name_to_index.get("humn").copied(),
    )
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Monkey {
    name: String,
    op: Op,
}

impl Monkey {
    #[inline]
    fn get_value(&self, monkeys: &[Monkey]) -> color_eyre::Result<i64> {
        self.op.get_value(monkeys).wrap_err_with(|| {
            format!(
                "Could not get value for monkey {} ({})",
                monkeys
                    .iter()
                    .find_position(|other| *other == self)
                    .unwrap()
                    .0,
                self,
            )
        })
    }

    #[inline]
    fn get_expected_lhs(
        &self,
        expected_result: i64,
        monkeys: &[Monkey],
    ) -> color_eyre::Result<i64> {
        self.op
            .get_expected_lhs(expected_result, monkeys)
            .wrap_err_with(|| {
                format!(
                    "Could not get lhs value for monkey {} ({}) given expected result {}",
                    monkeys
                        .iter()
                        .find_position(|other| *other == self)
                        .unwrap()
                        .0,
                    self,
                    expected_result,
                )
            })
    }

    #[inline]
    fn get_expected_rhs(
        &self,
        expected_result: i64,
        monkeys: &[Monkey],
    ) -> color_eyre::Result<i64> {
        self.op
            .get_expected_rhs(expected_result, monkeys)
            .wrap_err_with(|| {
                format!(
                    "Could not get rhs value for monkey {} ({}) given expected result {}",
                    monkeys
                        .iter()
                        .find_position(|other| *other == self)
                        .unwrap()
                        .0,
                    self,
                    expected_result,
                )
            })
    }
}

impl fmt::Display for Monkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.op)
    }
}

trait MonkeyCollection<Idx>
where
    Idx: ?Sized,
{
    fn get_value(&self, index: Idx) -> color_eyre::Result<i64>;
    fn get_expected_lhs(&self, index: usize, expected_result: i64) -> color_eyre::Result<i64>;
    fn get_expected_rhs(&self, index: usize, expected_result: i64) -> color_eyre::Result<i64>;
}

impl<T> MonkeyCollection<usize> for T
where
    T: AsRef<[Monkey]>,
{
    #[inline]
    fn get_value(&self, index: usize) -> color_eyre::Result<i64> {
        let self_ref = self.as_ref();
        self_ref[index].get_value(self_ref)
    }

    #[inline]
    fn get_expected_lhs(&self, index: usize, expected_result: i64) -> color_eyre::Result<i64> {
        let self_ref = self.as_ref();
        self_ref[index].get_expected_lhs(expected_result, self_ref)
    }

    #[inline]
    fn get_expected_rhs(&self, index: usize, expected_result: i64) -> color_eyre::Result<i64> {
        let self_ref = self.as_ref();
        self_ref[index].get_expected_rhs(expected_result, self_ref)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Op {
    Const(i64),
    Add(MonkeyRef, MonkeyRef),
    Sub(MonkeyRef, MonkeyRef),
    Mul(MonkeyRef, MonkeyRef),
    Div(MonkeyRef, MonkeyRef),
}

impl Op {
    fn get_value(&self, monkeys: &[Monkey]) -> color_eyre::Result<i64> {
        use MonkeyRef::*;
        use Op::*;

        match self {
            Const(num) => Ok(*num),

            Add(Resolved(lhs_idx), Resolved(rhs_idx)) => {
                Ok(monkeys.get_value(*lhs_idx)? + monkeys.get_value(*rhs_idx)?)
            }
            Sub(Resolved(lhs_idx), Resolved(rhs_idx)) => {
                Ok(monkeys.get_value(*lhs_idx)? - monkeys.get_value(*rhs_idx)?)
            }
            Mul(Resolved(lhs_idx), Resolved(rhs_idx)) => {
                Ok(monkeys.get_value(*lhs_idx)? * monkeys.get_value(*rhs_idx)?)
            }
            Div(Resolved(lhs_idx), Resolved(rhs_idx)) => {
                Ok(monkeys.get_value(*lhs_idx)? / monkeys.get_value(*rhs_idx)?)
            }

            Add(lhs, rhs) | Sub(lhs, rhs) | Mul(lhs, rhs) | Div(lhs, rhs) => {
                color_eyre::eyre::bail!(
                    "Operation has unresolved references (lhs = {lhs}, rhs = {rhs})"
                )
            }
        }
    }

    fn get_expected_lhs(
        &self,
        expected_result: i64,
        monkeys: &[Monkey],
    ) -> color_eyre::Result<i64> {
        use MonkeyRef::*;
        use Op::*;

        match self {
            Const(_) => color_eyre::eyre::bail!(
                "Cannot get expected (aka \"opposite\") lhs value for a constant operation"
            ),

            Add(_, Resolved(rhs_idx)) => Ok(expected_result - monkeys.get_value(*rhs_idx)?),
            Sub(_, Resolved(rhs_idx)) => Ok(expected_result + monkeys.get_value(*rhs_idx)?),
            Mul(_, Resolved(rhs_idx)) => Ok(expected_result / monkeys.get_value(*rhs_idx)?),
            Div(_, Resolved(rhs_idx)) => Ok(expected_result * monkeys.get_value(*rhs_idx)?),

            Add(lhs, rhs) | Sub(lhs, rhs) | Mul(lhs, rhs) | Div(lhs, rhs) => {
                color_eyre::eyre::bail!(
                    "Operation has unresolved references (lhs = {lhs}, rhs = {rhs})"
                )
            }
        }
    }

    fn get_expected_rhs(
        &self,
        expected_result: i64,
        monkeys: &[Monkey],
    ) -> color_eyre::Result<i64> {
        use MonkeyRef::*;
        use Op::*;

        match self {
            Const(_) => color_eyre::eyre::bail!(
                "Cannot get expected (aka \"opposite\") rhs value for constant operation `{self:?}`"
            ),

            Add(Resolved(lhs_idx), _) => Ok(expected_result - monkeys.get_value(*lhs_idx)?),
            Sub(Resolved(lhs_idx), _) => Ok(monkeys.get_value(*lhs_idx)? - expected_result),
            Mul(Resolved(lhs_idx), _) => Ok(expected_result / monkeys.get_value(*lhs_idx)?),
            Div(Resolved(lhs_idx), _) => Ok(monkeys.get_value(*lhs_idx)? / expected_result),

            Add(lhs, rhs) | Sub(lhs, rhs) | Mul(lhs, rhs) | Div(lhs, rhs) => {
                color_eyre::eyre::bail!(
                    "Operation has unresolved references (lhs = {lhs}, rhs = {rhs})"
                )
            }
        }
    }

    fn lhs_ref_unresolved(&self) -> bool {
        matches!(
            self,
            Self::Add(MonkeyRef::Unresolved(_), _)
                | Self::Sub(MonkeyRef::Unresolved(_), _)
                | Self::Mul(MonkeyRef::Unresolved(_), _)
                | Self::Div(MonkeyRef::Unresolved(_), _)
        )
    }

    fn rhs_ref_unresolved(&self) -> bool {
        matches!(
            self,
            Self::Add(_, MonkeyRef::Unresolved(_))
                | Self::Sub(_, MonkeyRef::Unresolved(_))
                | Self::Mul(_, MonkeyRef::Unresolved(_))
                | Self::Div(_, MonkeyRef::Unresolved(_))
        )
    }

    fn monkey_refs(&self) -> Option<(&MonkeyRef, &MonkeyRef)> {
        match self {
            Self::Add(lhs, rhs)
            | Self::Sub(lhs, rhs)
            | Self::Mul(lhs, rhs)
            | Self::Div(lhs, rhs) => Some((lhs, rhs)),

            Self::Const(_) => None,
        }
    }

    fn set_lhs(&mut self, lhs: MonkeyRef) -> Option<MonkeyRef> {
        match self {
            Self::Add(old_lhs, _)
            | Self::Sub(old_lhs, _)
            | Self::Mul(old_lhs, _)
            | Self::Div(old_lhs, _) => Some(std::mem::replace(old_lhs, lhs)),

            Self::Const(_) => None,
        }
    }

    fn set_rhs(&mut self, rhs: MonkeyRef) -> Option<MonkeyRef> {
        match self {
            Self::Add(_, old_rhs)
            | Self::Sub(_, old_rhs)
            | Self::Mul(_, old_rhs)
            | Self::Div(_, old_rhs) => Some(std::mem::replace(old_rhs, rhs)),

            Self::Const(_) => None,
        }
    }
}

impl Default for Op {
    fn default() -> Self {
        Self::Const(Default::default())
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Const(n) => n.fmt(f),
            Self::Add(m1, m2) => write!(f, "{m1} + {m2}"),
            Self::Sub(m1, m2) => write!(f, "{m1} - {m2}"),
            Self::Mul(m1, m2) => write!(f, "{m1} * {m2}"),
            Self::Div(m1, m2) => write!(f, "{m1} / {m2}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MonkeyRef {
    Unresolved(String),
    Resolved(usize),
}

impl MonkeyRef {
    fn unresolved_name(&self) -> Option<&str> {
        match self {
            MonkeyRef::Unresolved(name) => Some(name.as_str()),
            MonkeyRef::Resolved(_) => None,
        }
    }

    fn resolved_idx(&self) -> Option<usize> {
        match self {
            MonkeyRef::Unresolved(_) => None,
            MonkeyRef::Resolved(idx) => Some(*idx),
        }
    }
}

impl Default for MonkeyRef {
    fn default() -> Self {
        Self::Unresolved(Default::default())
    }
}

impl fmt::Display for MonkeyRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unresolved(s) => s.fmt(f),
            Self::Resolved(idx) => write!(f, "{{{idx}}}"),
        }
    }
}

super::challenge_solver_test_boilerplate! {
    Solver21;
    "root: pppw + sjmn\n\
     dbpl: 5\n\
     cczh: sllz + lgvd\n\
     zczc: 2\n\
     ptdq: humn - dvpt\n\
     dvpt: 3\n\
     lfqf: 4\n\
     humn: 5\n\
     ljgn: 2\n\
     sjmn: drzm * dbpl\n\
     sllz: 4\n\
     pppw: cczh / lfqf\n\
     lgvd: ljgn * ptdq\n\
     drzm: hmdt - zczc\n\
     hmdt: 32" => {
        a as i64: 152,
        b as i64: 301,
    }

    const OPS_TEST_INPUT: &str = "\
        aaaa: aaab + aaac\n\
        bbbb: bbbc - bbbd\n\
        cccc: cccd * ccce\n\
        dddd: ddde / dddf\n\
        \n\
        aaab: 3\n\
        aaac: 2\n\
        \n\
        bbbc: 2\n\
        bbbd: 5\n\
        \n\
        cccd: 12\n\
        ccce: -3\n\
        \n\
        ddde: 20\n\
        dddf: -4\n\
    ";


    #[test]
    fn ops_get_value() -> color_eyre::Result<()> {
        color_eyre::install()?;

        let mut monkeys = parse::parse_input(OPS_TEST_INPUT)?;
        resolve_monkeys(&mut monkeys);

        assert_eq!(monkeys[0].op.get_value(&monkeys)?, 3 + 2, "Op::get_value() addition failed");
        assert_eq!(monkeys[1].op.get_value(&monkeys)?, 2 - 5, "Op::op.get_value() subtraction failed");
        assert_eq!(monkeys[2].op.get_value(&monkeys)?, 12 * -3, "Op::op.get_value() multiplication failed");
        assert_eq!(monkeys[3].op.get_value(&monkeys)?, 20 / -4, "Op::op.get_value() division failed");

        assert_eq!(monkeys[0].get_value(&monkeys)?, 3 + 2, "Monkey::get_value() addition failed");
        assert_eq!(monkeys[1].get_value(&monkeys)?, 2 - 5, "Monkey::get_value() subtraction failed");
        assert_eq!(monkeys[2].get_value(&monkeys)?, 12 * -3, "Monkey::get_value() multiplication failed");
        assert_eq!(monkeys[3].get_value(&monkeys)?, 20 / -4, "Monkey::get_value() division failed");

        assert_eq!(monkeys.get_value(0)?, 3 + 2, "MonkeyCollection::get_value() addition failed");
        assert_eq!(monkeys.get_value(1)?, 2 - 5, "MonkeyCollection::get_value() subtraction failed");
        assert_eq!(monkeys.get_value(2)?, 12 * -3, "MonkeyCollection::get_value() multiplication failed");
        assert_eq!(monkeys.get_value(3)?, 20 / -4, "MonkeyCollection::get_value() division failed");

        Ok(())
    }

    #[test]
    fn ops_get_expected_lhs() -> color_eyre::Result<()> {
        color_eyre::install()?;

        let mut monkeys = parse::parse_input(OPS_TEST_INPUT)?;
        resolve_monkeys(&mut monkeys);

        assert_eq!(monkeys[0].op.get_expected_lhs(7, &monkeys)?, 5, "Op::get_expected_lhs() addition failed");
        assert_eq!(monkeys[1].op.get_expected_lhs(-42, &monkeys)?, -37, "Op::get_expected_lhs() subtraction failed");
        assert_eq!(monkeys[2].op.get_expected_lhs(27, &monkeys)?, -9, "Op::get_expected_lhs() multiplication failed");
        assert_eq!(monkeys[3].op.get_expected_lhs(-16, &monkeys)?, 64, "Op::get_expected_lhs() division failed");

        assert_eq!(monkeys[0].get_expected_lhs(7, &monkeys)?, 5, "Monkey::get_expected_lhs() addition failed");
        assert_eq!(monkeys[1].get_expected_lhs(-42, &monkeys)?, -37, "Monkey::get_expected_lhs() subtraction failed");
        assert_eq!(monkeys[2].get_expected_lhs(27, &monkeys)?, -9, "Monkey::get_expected_lhs() multiplication failed");
        assert_eq!(monkeys[3].get_expected_lhs(-16, &monkeys)?, 64, "Monkey::get_expected_lhs() division failed");

        assert_eq!(monkeys.get_expected_lhs(0, 7)?, 5, "MonkeyCollection::get_expected_lhs() addition failed");
        assert_eq!(monkeys.get_expected_lhs(1, -42)?, -37, "MonkeyCollection::get_expected_lhs() subtraction failed");
        assert_eq!(monkeys.get_expected_lhs(2, 27)?, -9, "MonkeyCollection::get_expected_lhs() multiplication failed");
        assert_eq!(monkeys.get_expected_lhs(3, -16)?, 64, "MonkeyCollection::get_expected_lhs() division failed");

        Ok(())
    }

    #[test]
    fn ops_get_expected_rhs() -> color_eyre::Result<()> {
        color_eyre::install()?;

        let mut monkeys = parse::parse_input(OPS_TEST_INPUT)?;
        resolve_monkeys(&mut monkeys);

        assert_eq!(monkeys[0].op.get_expected_rhs(-14, &monkeys)?, -17, "Op::get_expected_lhs() addition failed");
        assert_eq!(monkeys[1].op.get_expected_rhs(10, &monkeys)?, -8, "Op::get_expected_lhs() subtraction failed");
        assert_eq!(monkeys[2].op.get_expected_rhs(24, &monkeys)?, 2, "Op::get_expected_lhs() multiplication failed");
        assert_eq!(monkeys[3].op.get_expected_rhs(4, &monkeys)?, 5, "Op::get_expected_lhs() division failed");

        assert_eq!(monkeys[0].get_expected_rhs(-14, &monkeys)?, -17, "Monkey::get_expected_lhs() addition failed");
        assert_eq!(monkeys[1].get_expected_rhs(10, &monkeys)?, -8, "Monkey::get_expected_lhs() subtraction failed");
        assert_eq!(monkeys[2].get_expected_rhs(24, &monkeys)?, 2, "Monkey::get_expected_lhs() multiplication failed");
        assert_eq!(monkeys[3].get_expected_rhs(4, &monkeys)?, 5, "Monkey::get_expected_lhs() division failed");

        assert_eq!(monkeys.get_expected_rhs(0, -14)?, -17, "MonkeyCollection::get_expected_lhs() addition failed");
        assert_eq!(monkeys.get_expected_rhs(1, 10)?, -8, "MonkeyCollection::get_expected_lhs() subtraction failed");
        assert_eq!(monkeys.get_expected_rhs(2, 24)?, 2, "MonkeyCollection::get_expected_lhs() multiplication failed");
        assert_eq!(monkeys.get_expected_rhs(3, 4)?, 5, "MonkeyCollection::get_expected_lhs() division failed");

        Ok(())
    }
}
