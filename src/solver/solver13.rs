use std::{
    cmp::{self, Ordering},
    fmt,
    fs::File,
    io::{BufRead, BufReader, Read},
};

use color_eyre::eyre::Context;
use serde::Deserialize;

use super::ChallengeSolver;

#[derive(Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
enum Node {
    Number(u64),
    List(Vec<Node>),
}

impl Node {
    fn with_slice<T>(&self, f: impl FnOnce(&[Node]) -> T) -> T {
        match self {
            Self::List(l) => f(&l[..]),
            Self::Number(n) => f(&[Self::Number(*n)]),
        }
    }
}

impl cmp::PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Node::Number(a), Node::Number(b)) => a.partial_cmp(b),

            (l, r) => Some(l.with_slice(|l| {
                r.with_slice(|r| {
                    l.iter()
                        .zip(r.iter())
                        .map(|(aa, bb)| aa.cmp(bb))
                        // return the first ordering that isn't `Equal`
                        .find(|&ord| ord != Ordering::Equal)
                        // or compare the lengths
                        .unwrap_or_else(|| l.len().cmp(&r.len()))
                })
            })),
        }
    }
}

impl cmp::Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::List(l) => f.debug_list().entries(l).finish(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Solver13;

impl ChallengeSolver for Solver13 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        13
    }

    fn solve_a(&mut self, mut input: BufReader<File>) -> color_eyre::Result<()> {
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;

        let mut sum = 0;

        for (i, groups) in input_buf.split("\n\n").enumerate() {
            let i = i + 1;

            let mut nodes = groups
                .lines()
                .map(|line| serde_json::from_str::<Node>(line).unwrap());
            let l = nodes.next().unwrap();
            let r = nodes.next().unwrap();

            println!("\n== Pair {i} ==");
            println!("l = {l:?}");
            println!("r = {r:?}");
            println!("l < r = {}", l < r);

            if l < r {
                sum += i;
            }
        }

        println!("\n---\n\nsum = {sum}");

        Ok(())
    }

    fn solve_b(&mut self, input: BufReader<File>) -> color_eyre::Result<()> {
        let dividers = vec![
            Node::List(vec![Node::Number(2)]),
            Node::List(vec![Node::Number(6)]),
        ];

        let mut packets = input
            .lines()
            .map(|s| s.unwrap())
            .filter(|s| !s.is_empty())
            .map(|line| serde_json::from_str::<Node>(&line).unwrap())
            .chain(dividers.iter().cloned())
            .collect::<Vec<_>>();

        packets.sort();

        let decoder_key = dividers
            .iter()
            .map(|d| packets.binary_search(d).unwrap() + 1)
            .product::<usize>();

        println!("decoder_key = {decoder_key}");

        Ok(())
    }
}
