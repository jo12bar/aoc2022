use std::{collections::HashMap, io::BufRead};

use color_eyre::eyre::Context;
use itertools::Itertools;

use self::{
    namemap::NameMap,
    parse::{Name, Valve},
};

use super::ChallengeSolver;

mod namemap;
mod parse;

#[derive(Debug, Default)]
pub struct Solver16;

impl ChallengeSolver for Solver16 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        16
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;

        let net = Network::new(&input_buf)?;
        let state = State {
            net: &net,
            position: Name(*b"AA"),
            max_turns: 30,
            turn: 0,
            pressure: 0,
            open_valves: Default::default(),
        };

        let mut best = Best::default();
        let state = state.apply_best_moves(&mut best);
        println!("final_pressure = {}", state.pressure);

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;

        let net = Network::new(&input_buf)?;
        let state = State {
            net: &net,
            position: Name(*b"AA"),
            max_turns: 26,
            turn: 0,
            pressure: 0,
            open_valves: Default::default(),
        };

        let mut best = Best::default();
        state.apply_best_moves(&mut best);

        let best_pressure = best
            .iter()
            .tuple_combinations()
            .filter(|(human, elephant)| human.0.is_disjoint(elephant.0))
            .map(|(human, elephant)| human.1 + elephant.1)
            .max()
            .unwrap();

        println!("final_pressure = {best_pressure}");

        Ok(Box::new(()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
struct Flow(u64);

type Path = Vec<(Name, Name)>;
type Connections = NameMap<(Path, Flow)>;
type Best = HashMap<NameMap<()>, u64>;

struct Network {
    valves: NameMap<(Valve, Connections)>,
}

impl Network {
    fn new(input: &str) -> Result<Self, NetworkError> {
        let mut net = Self {
            valves: parse::parse_input(input)?
                .into_iter()
                // Start off with zero connections (since we're still parsing)
                .map(|valve| (valve.name, (valve, Connections::default())))
                .collect(),
        };

        let names = net.valves.keys().collect::<Vec<_>>();

        for name in names {
            // Fill in the connections as needed
            let conns = net.connections(name);
            net.valves.get_mut(name).unwrap().1 = conns;
        }

        Ok(net)
    }

    /// Given a valve name, return a list of valves we can travel to, along
    /// with the path to get there, and their flow.
    ///
    /// Only the shortest paths are considered, so the search ends.
    fn connections(&self, start: Name) -> Connections {
        let mut current = Connections::default();
        {
            let valve = &self.valves.get(start).unwrap().0;
            current.insert(start, (vec![], Flow(valve.flow)));
        }

        let mut connections = current.clone();

        while !current.is_empty() {
            let mut next = Connections::default();

            for (name, (path, _flow)) in current.iter() {
                for link in self.valves.get(name).unwrap().0.links.iter().copied() {
                    let valve = &self.valves.get(link).unwrap().0;

                    if !connections.contains(link) {
                        let conn_path: Path = path
                            .iter()
                            .copied()
                            .chain(std::iter::once((name, link)))
                            .collect();

                        let item = (conn_path.clone(), Flow(valve.flow));
                        connections.insert(link, item.clone());
                        next.insert(link, item);
                    }
                }
            }

            current = next;
        }

        connections
    }
}

#[derive(Debug, thiserror::Error)]
enum NetworkError {
    #[error("Could not parse challenge input into a valve network")]
    BadInput {
        #[from]
        source: parse::ParseInputError,
    },
}

#[derive(Debug, Clone)]
struct Move<'a> {
    reward: u64,
    target: Name,
    path: &'a Path,
}

impl Move<'_> {
    fn cost(&self) -> u64 {
        let travel_turns = self.path.len() as u64;
        let open_turns = 1_u64;
        travel_turns + open_turns
    }
}

#[derive(Clone)]
struct State<'a> {
    net: &'a Network,
    position: Name,
    max_turns: u64,
    turn: u64,
    pressure: u64,
    open_valves: NameMap<()>,
}

impl State<'_> {
    fn turns_left(&self) -> u64 {
        self.max_turns - self.turn
    }

    /// Compute all moves and expected reward (pressure contributed till time
    /// runs out if we travel to it and open it now)
    fn moves(&self) -> impl Iterator<Item = Move> + '_ {
        let (_valves, connections) = &self.net.valves.get(self.position).unwrap();
        connections.iter().filter_map(|(name, (path, flow))| {
            if self.open_valves.contains(name) {
                return None;
            }

            if flow.0 == 0 {
                return None;
            }

            let travel_turns = path.len() as u64;
            let open_turns = 1_u64;
            let turns_spent_open = self.turns_left().checked_sub(travel_turns + open_turns)?;
            let reward = flow.0 * turns_spent_open;

            Some(Move {
                reward,
                target: name,
                path,
            })
        })
    }

    // fn find_best_moves(&self) -> (Self, Vec<Move>) {
    //     let mut best_moves = vec![];
    //     let mut best_state = self.clone();

    //     for mv in self.moves() {
    //         let next = self.apply(&mv);
    //         let (next, mut next_moves) = next.find_best_moves();
    //         next_moves.push(mv);
    //         if next.pressure > best_state.pressure {
    //             best_moves = next_moves;
    //             best_state = next;
    //         }
    //     }

    //     (best_state, best_moves)
    // }

    fn apply_best_moves(&self, best: &mut Best) -> Self {
        let mut best_state = self.clone();

        best.entry(self.open_valves.clone())
            .and_modify(|v| {
                if self.pressure > *v {
                    *v = self.pressure;
                }
            })
            .or_insert(self.pressure);

        for mv in self.moves() {
            let next = self.apply(&mv).apply_best_moves(best);
            if next.pressure > best_state.pressure {
                best_state = next;
            }
        }

        best_state
    }

    /// Apply a given move
    fn apply(&self, mv: &Move) -> Self {
        let mut next = self.clone();
        next.position = mv.target;
        next.turn += mv.cost();
        next.pressure += mv.reward;
        next.open_valves.insert(mv.target, ());
        next
    }
}
