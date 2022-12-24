use std::{
    fmt,
    fs::File,
    io::{BufRead, BufReader},
};

use color_eyre::eyre::Context;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space1,
    combinator::{all_consuming, map, value},
    sequence::preceded,
    Finish, IResult,
};

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver10;

impl ChallengeSolver for Solver10 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        10
    }

    fn solve_a(&mut self, input: BufReader<File>) -> color_eyre::Result<()> {
        // Parse instructions
        let instructions = input
            .lines()
            .map(|l| -> color_eyre::Result<Instruction> {
                l.wrap_err("Could not read line from input file")
                    .map(|l| all_consuming(Instruction::parse)(&l).finish().unwrap().1)
            })
            .collect::<Result<Vec<Instruction>, _>>()
            .wrap_err("Could not parse instructions")?;

        // Execute instructions
        println!("=============");
        println!("| EXECUTION |");
        println!("=============");
        let mut machine = Machine::new(instructions);

        let mut total = 0;
        let mut count = 0;

        loop {
            println!("{machine:?}");

            if matches!(machine.cpu.cycle, 20 | 60 | 100 | 140 | 180 | 220) {
                total += machine.cpu.cycle as i64 * machine.cpu.x as i64;
                count += 1;
                println!(
                    "CYCLE: {}, X: {}, STRENGTH: {}, TOTAL: {}",
                    machine.cpu.cycle,
                    machine.cpu.x,
                    machine.cpu.cycle as i64 * machine.cpu.x as i64,
                    total,
                );
            }

            if !machine.tick() {
                break;
            }
        }

        // Display the result
        println!("\n==========");
        println!("| RESULT |");
        println!("==========");
        println!("total: {total}");
        println!("interesting count: {count}");

        Ok(())
    }

    fn solve_b(&mut self, input: BufReader<File>) -> color_eyre::Result<()> {
        // Parse instructions
        let instructions = input
            .lines()
            .map(|l| -> color_eyre::Result<Instruction> {
                l.wrap_err("Could not read line from input file")
                    .map(|l| all_consuming(Instruction::parse)(&l).finish().unwrap().1)
            })
            .collect::<Result<Vec<Instruction>, _>>()
            .wrap_err("Could not parse instructions")?;

        // Execute instructions
        println!("=============");
        println!("| EXECUTION |");
        println!("=============");
        let mut machine = Machine::new(instructions);

        loop {
            machine.draw();
            println!("{machine:?}");
            if !machine.tick() {
                break;
            }
        }

        Ok(())
    }
}

struct Machine {
    instructions: Vec<Instruction>,
    cpu: Cpu,
    display: CrtDisplay,
}

impl Machine {
    fn new(instructions: Vec<Instruction>) -> Self {
        let cpu = Cpu::new(&instructions);
        Self {
            instructions,
            cpu,
            display: CrtDisplay::default(),
        }
    }

    fn tick(&mut self) -> bool {
        self.cpu.execute(&self.instructions)
    }

    fn draw(&mut self) {
        self.display.draw(self.cpu.cycle as _, self.cpu.x);
    }
}

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "cycle={:<3} pc={:<3} x={:<3} cur_ins={:?}",
            self.cpu.cycle, self.cpu.pc, self.cpu.x, self.cpu.cur_ins
        )?;
        self.display.fmt(f)
    }
}

#[derive(Debug)]
struct Cpu {
    x: i32,
    pc: usize,
    cycle: usize,
    cur_ins: Option<(Instruction, u8)>,
}

impl Cpu {
    fn new(instructions: &[Instruction]) -> Self {
        let mut this = Self {
            x: 1,
            pc: 0,
            cycle: 1,
            cur_ins: None,
        };
        this.decode(instructions);
        this
    }

    fn decode(&mut self, instructions: &[Instruction]) {
        self.cur_ins = instructions.get(self.pc).map(|ins| (*ins, ins.cycles()));
        self.pc += 1;
    }

    fn execute(&mut self, instructions: &[Instruction]) -> bool {
        if self.cur_ins.is_none() {
            return false;
        }

        let (ins, cycles_left) = self.cur_ins.as_mut().unwrap();
        *cycles_left -= 1;

        if *cycles_left == 0 {
            match ins {
                Instruction::Noop => {}
                Instruction::AddX(x) => self.x += *x,
            }
            self.decode(instructions);
        }

        self.cycle += 1;

        true
    }
}

struct CrtDisplay {
    display_lines: Vec<u64>,
}

impl CrtDisplay {
    fn new() -> Self {
        Self {
            display_lines: Vec::new(),
        }
    }

    fn draw(&mut self, cycle: u64, x: i32) {
        let cycle = cycle - 1;
        let crt_line = (cycle / 40) as usize;
        if crt_line + 1 > self.display_lines.len() {
            self.display_lines.push(0);
        }
        let crt_line = self.display_lines.get_mut(crt_line).unwrap();
        let cycle_mask = cycle_mask(cycle);
        let sprite = sprite_value(x as _);
        *crt_line |= cycle_mask & sprite;
    }
}

impl Default for CrtDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CrtDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "╭──────────────────────────────────────────╮")?;
        for line in &self.display_lines {
            write!(f, "│ ")?;
            for i in 0..40 {
                let c = if line & cycle_mask(i) > 0 { '█' } else { ' ' };
                write!(f, "{c}")?;
            }
            writeln!(f, " │")?;
        }
        write!(f, "╰──────────────────────────────────────────╯")
    }
}

const DISPLAY_MASK: u64 = 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111;

fn sprite_value(pos: i32) -> u64 {
    let model = 0b1_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_u64;
    let shifted;
    if pos < 0 {
        (shifted, _) = model.overflowing_shl((-pos).try_into().unwrap());
    } else {
        (shifted, _) = model.overflowing_shr(pos.try_into().unwrap());
    }
    shifted & DISPLAY_MASK
}

fn cycle_mask(cycle: u64) -> u64 {
    (0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000 >> (cycle % 40)) & DISPLAY_MASK
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Instruction {
    Noop,
    AddX(i32),
}

impl Instruction {
    fn parse_noop(i: &str) -> IResult<&str, Self> {
        value(Self::Noop, tag("noop"))(i)
    }

    fn parse_add_reg(i: &str) -> IResult<&str, Self> {
        map(
            preceded(tag("addx"), preceded(space1, nom::character::complete::i32)),
            Self::AddX,
        )(i)
    }

    /// Try to parse an instruction
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((Self::parse_noop, Self::parse_add_reg))(i)
    }

    /// Get the number of cycles that this instruction should be executed for.
    fn cycles(&self) -> u8 {
        match self {
            Instruction::Noop => 1,
            Instruction::AddX(_) => 2,
        }
    }
}

#[test]
fn test_sprite_value() {
    assert_eq!(
        format!("{:040b}", sprite_value(0)),
        "1100000000000000000000000000000000000000"
    );
    assert_eq!(
        format!("{:040b}", sprite_value(1)),
        "1110000000000000000000000000000000000000"
    );
    assert_eq!(
        format!("{:040b}", sprite_value(38)),
        "0000000000000000000000000000000000000111"
    );
    assert_eq!(
        format!("{:040b}", sprite_value(39)),
        "0000000000000000000000000000000000000011"
    );
    assert_eq!(
        format!("{:040b}", sprite_value(40)),
        "0000000000000000000000000000000000000001"
    );
    assert_eq!(
        format!("{:040b}", sprite_value(-1)),
        "1000000000000000000000000000000000000000"
    );
}
