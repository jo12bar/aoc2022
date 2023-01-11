use std::io::BufRead;

use camino::Utf8PathBuf;
use color_eyre::eyre::Context;
use id_tree::{InsertBehavior, Node, NodeId, Tree};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::{all_consuming, map},
    sequence::{preceded, separated_pair},
    Finish, IResult,
};

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver07;

impl ChallengeSolver for Solver07 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        7
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut vm = Vm::new().wrap_err("Couldn't create VM")?;

        for line in input.lines() {
            let line = line?;

            let parsed = all_consuming(parse_line)(&line).finish().unwrap().1;

            match parsed {
                Line::Command(cmd) => match cmd {
                    Command::Ls => {} // Just ignore ls

                    Command::Cd(path) => match path.as_str() {
                        // We start in `/`, and we never go back to it. So just ignore it.
                        "/" => {}

                        ".." => {
                            vm.cd_parent_dir()
                                .wrap_err("Couldn't `cd` into parent directory")?;
                        }

                        _ => {
                            vm.cd(&path)
                                .wrap_err("Couldn't `cd` into a child directory")?;
                        }
                    },
                },

                Line::Entry(entry) => {
                    vm.add_entry(entry)
                        .wrap_err("Couldn't add entry to VM's file tree")?;
                }
            }
        }

        let mut s = String::new();
        vm.tree.write_formatted(&mut s)?;
        println!("{s}");

        println!(
            "\nComputing sum of sizes of all dirs with individual sizes of at most 100_000..."
        );
        let sum = vm
            .tree
            .traverse_pre_order(vm.tree.root_node_id().unwrap())?
            // only consider directories with children!
            .filter(|d| !d.children().is_empty())
            .map(|d| total_size(&vm.tree, d).unwrap())
            .filter(|&s| s <= 100_000)
            .inspect(|s| {
                dbg!(s);
            })
            .sum::<u64>();

        println!("\nDone! Sum of sizes = {sum}");

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let mut vm = Vm::new().wrap_err("Couldn't create VM")?;

        for line in input.lines() {
            let line = line?;

            let parsed = all_consuming(parse_line)(&line).finish().unwrap().1;

            match parsed {
                Line::Command(cmd) => match cmd {
                    Command::Ls => {} // Just ignore ls

                    Command::Cd(path) => match path.as_str() {
                        // We start in `/`, and we never go back to it. So just ignore it.
                        "/" => {}

                        ".." => {
                            vm.cd_parent_dir()
                                .wrap_err("Couldn't `cd` into parent directory")?;
                        }

                        _ => {
                            vm.cd(&path)
                                .wrap_err("Couldn't `cd` into a child directory")?;
                        }
                    },
                },

                Line::Entry(entry) => {
                    vm.add_entry(entry)
                        .wrap_err("Couldn't add entry to VM's file tree")?;
                }
            }
        }

        let mut s = String::new();
        vm.tree.write_formatted(&mut s)?;
        println!("{s}");

        const NEEDED_FREE_SPACE: u64 = 30_000_000;

        let used_space = total_size(&vm.tree, vm.tree.get(vm.tree.root_node_id().unwrap())?)?;
        let free_space = TOTAL_SPACE.checked_sub(used_space).unwrap();
        let minimum_space_to_free = NEEDED_FREE_SPACE.checked_sub(free_space).unwrap();

        println!("Total space:          {TOTAL_SPACE:>8}");
        println!("Used space:           {used_space:>8}");
        println!("Free space:           {free_space:>8}");
        println!("Min. required space:  {NEEDED_FREE_SPACE:>8}");
        println!("Min. space to free:   {minimum_space_to_free:>8}\n");

        let (removed_dir_size, dir_to_remove) = vm
            .tree
            .traverse_pre_order(vm.tree.root_node_id().unwrap())?
            // only consider directories with children!
            .filter(|d| !d.children().is_empty())
            .map(|d| (total_size(&vm.tree, d).unwrap(), d))
            .filter(|(s, _)| *s >= minimum_space_to_free)
            .inspect(|s| {
                dbg!(s.0);
            })
            .reduce(|acc, d| if acc.0 <= d.0 { acc } else { d })
            .unwrap();

        println!("\nFound directory of size {removed_dir_size} to remove!");
        println!("(path: {})", dir_to_remove.data().path);

        Ok(Box::new(()))
    }
}

///////////////////////// VIRTUAL MACHINE

const TOTAL_SPACE: u64 = 70_000_000;

#[derive(Debug)]
struct FsEntry {
    path: Utf8PathBuf,
    size: u64,
}

fn total_size(tree: &Tree<FsEntry>, node: &Node<FsEntry>) -> color_eyre::Result<u64> {
    let mut total = node.data().size;
    for child in node.children() {
        total += total_size(tree, tree.get(child)?)?;
    }
    Ok(total)
}

#[derive(Debug)]
struct Vm {
    tree: Tree<FsEntry>,
    pwd: NodeId,
}

impl Vm {
    fn new() -> color_eyre::Result<Self> {
        let mut tree = Tree::new();
        let root = tree.insert(
            Node::new(FsEntry {
                path: "/".into(),
                size: 0,
            }),
            InsertBehavior::AsRoot,
        )?;
        Ok(Self { tree, pwd: root })
    }

    fn cd_parent_dir(&mut self) -> color_eyre::Result<()> {
        self.pwd = self
            .tree
            .get(&self.pwd)?
            .parent()
            .ok_or(color_eyre::eyre::eyre!(
                "Tried to cd to parent when pwd is already `/`"
            ))?
            .clone();
        Ok(())
    }

    fn cd(&mut self, path: &Utf8PathBuf) -> color_eyre::Result<()> {
        let node = Node::new(FsEntry {
            path: path.clone(),
            size: 0,
        });
        self.pwd = self
            .tree
            .insert(node, InsertBehavior::UnderNode(&self.pwd))?;
        Ok(())
    }

    fn add_entry(&mut self, entry: Entry) -> color_eyre::Result<()> {
        match entry {
            Entry::Dir(_dir_path) => {
                // Ignore. This is handled when `cd`ing into directories.
            }

            Entry::File(size, path) => {
                let node = Node::new(FsEntry { size, path });
                self.tree
                    .insert(node, InsertBehavior::UnderNode(&self.pwd))?;
            }
        }

        Ok(())
    }
}

///////////////////////// PARSING INPUT

fn parse_path(i: &str) -> IResult<&str, Utf8PathBuf> {
    map(
        take_while1(|c: char| "abcdefghijklmnopqrstuvwxyz0123456789./".contains(c)),
        Into::into,
    )(i)
}

#[derive(Debug)]
struct Ls;

fn parse_ls(i: &str) -> IResult<&str, Ls> {
    map(tag("ls"), |_| Ls)(i)
}

#[derive(Debug)]
struct Cd(Utf8PathBuf);

fn parse_cd(i: &str) -> IResult<&str, Cd> {
    map(preceded(tag("cd "), parse_path), Cd)(i)
}

#[derive(Debug)]
enum Command {
    Ls,
    Cd(Utf8PathBuf),
}

impl From<Ls> for Command {
    fn from(_: Ls) -> Self {
        Self::Ls
    }
}

impl From<Cd> for Command {
    fn from(Cd(path): Cd) -> Self {
        Command::Cd(path)
    }
}

fn parse_command(i: &str) -> IResult<&str, Command> {
    let (i, _) = tag("$ ")(i)?;
    alt((map(parse_ls, Into::into), map(parse_cd, Into::into)))(i)
}

#[derive(Debug)]
enum Entry {
    Dir(Utf8PathBuf),
    File(u64, Utf8PathBuf),
}

fn parse_entry(i: &str) -> IResult<&str, Entry> {
    let parse_file = map(
        separated_pair(nom::character::complete::u64, tag(" "), parse_path),
        |(size, path)| Entry::File(size, path),
    );
    let parse_dir = map(preceded(tag("dir "), parse_path), Entry::Dir);

    alt((parse_file, parse_dir))(i)
}

#[derive(Debug)]
enum Line {
    Command(Command),
    Entry(Entry),
}

fn parse_line(i: &str) -> IResult<&str, Line> {
    alt((
        map(parse_command, Line::Command),
        map(parse_entry, Line::Entry),
    ))(i)
}
