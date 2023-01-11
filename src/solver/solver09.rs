use std::{
    collections::{HashSet, VecDeque},
    fmt,
    hash::Hash,
    io::{self, BufRead},
    ops,
    time::{Duration, Instant},
};

use color_eyre::eyre::Context;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        MouseEvent, MouseEventKind,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space1,
    combinator::{all_consuming, map, value},
    sequence::{preceded, tuple},
    Finish, IResult,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        canvas::{Canvas, Line, Points},
        Block, Borders, Paragraph,
    },
    Frame, Terminal,
};

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver09;

impl ChallengeSolver for Solver09 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        9
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        // setup terminal
        enable_raw_mode().wrap_err("Could not initialize terminal UI")?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .wrap_err("Could not initialize terminal UI")?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).wrap_err("Could not initialize terminal UI")?;

        // Initialize app
        let app = AppA::new(input)?;

        // Run the app
        let tick_rate = Duration::from_secs_f64(1.0 / 60.0);
        let res = app.run(&mut terminal, tick_rate);

        // Restore terminal
        disable_raw_mode().wrap_err("Could not deinitialize terminal UI")?;
        crossterm::execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .wrap_err("Could not deinitialize terminal UI")?;
        terminal
            .show_cursor()
            .wrap_err("Could not deinitialize terminal UI")?;

        // Remember to unwrap the result of running the app AFTER restoring the terminal
        res?;

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        // setup terminal
        enable_raw_mode().wrap_err("Could not initialize terminal UI")?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .wrap_err("Could not initialize terminal UI")?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).wrap_err("Could not initialize terminal UI")?;

        // Initialize app
        let app = AppB::new(input)?;

        // Run the app
        let tick_rate = Duration::from_secs_f64(1.0 / 60.0);
        let res = app.run(&mut terminal, tick_rate);

        // Restore terminal
        disable_raw_mode().wrap_err("Could not deinitialize terminal UI")?;
        crossterm::execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .wrap_err("Could not deinitialize terminal UI")?;
        terminal
            .show_cursor()
            .wrap_err("Could not deinitialize terminal UI")?;

        // Remember to unwrap the result of running the app AFTER restoring the terminal
        res?;

        Ok(Box::new(()))
    }
}

struct AppA {
    instructions: VecDeque<Instruction>,
    head: GridPos,
    tail: GridPos,
    tail_visited_positions: HashSet<GridPos>,
    instructions_scroll: u16,
}

impl AppA {
    fn new(input: &mut dyn BufRead) -> color_eyre::Result<Self> {
        let instructions = input
            .lines()
            .map(|l| -> color_eyre::Result<Instruction> {
                l.wrap_err("Could not read line from input file")
                    .map(|l| all_consuming(Instruction::parse)(&l).finish().unwrap().1)
            })
            .collect::<Result<VecDeque<Instruction>, _>>()
            .wrap_err("Could not parse instructions")?;

        Ok(Self {
            instructions,
            head: GridPos { x: 0, y: 0 },
            tail: GridPos { x: 0, y: 0 },
            tail_visited_positions: HashSet::default(),
            instructions_scroll: 0,
        })
    }

    /// Run the app.
    fn run<B: Backend>(
        mut self,
        terminal: &mut Terminal<B>,
        tick_rate: Duration,
    ) -> color_eyre::Result<()> {
        let mut last_tick = Instant::now();
        loop {
            terminal
                .draw(|f| self.ui(f))
                .wrap_err("Error while drawing UI frame.")?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).wrap_err("Could not poll terminal for new I/O events")? {
                match event::read().wrap_err("Could not read terminal I/O event")? {
                    Event::Key(key) => match key {
                        KeyEvent {
                            code: KeyCode::Char('q'),
                            ..
                        } => return Ok(()),

                        KeyEvent {
                            code: KeyCode::Up,
                            kind: KeyEventKind::Press | KeyEventKind::Repeat,
                            ..
                        } => {
                            self.scroll_up(1);
                        }

                        KeyEvent {
                            code: KeyCode::Down,
                            kind: KeyEventKind::Press | KeyEventKind::Repeat,
                            ..
                        } => {
                            self.scroll_down(1);
                        }

                        KeyEvent {
                            code: KeyCode::PageUp,
                            kind: KeyEventKind::Press | KeyEventKind::Repeat,
                            ..
                        } => {
                            self.scroll_up(10);
                        }

                        KeyEvent {
                            code: KeyCode::PageDown,
                            kind: KeyEventKind::Press | KeyEventKind::Repeat,
                            ..
                        } => {
                            self.scroll_down(10);
                        }

                        _ => {}
                    },

                    Event::Mouse(ev) => match ev {
                        MouseEvent {
                            kind: MouseEventKind::ScrollUp,
                            ..
                        } => {
                            self.scroll_up(2);
                        }

                        MouseEvent {
                            kind: MouseEventKind::ScrollDown,
                            ..
                        } => {
                            self.scroll_down(2);
                        }

                        _ => {}
                    },

                    _ => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }
        }
    }

    /// Render the app UI to a tui frame
    fn ui<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(f.size());

        let sidebar_chunks = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Percentage(30),
            ])
            .split(chunks[0]);

        // Render the current count of visited places
        let visited_block = Block::default()
            .title("Tail locations")
            .borders(Borders::ALL);
        let visited = Paragraph::new(Span::styled(
            self.tail_visited_positions.len().to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .block(visited_block);
        f.render_widget(visited, sidebar_chunks[0]);

        // Render out all the instructions
        let instructions_block = Block::default().title("Instructions").borders(Borders::ALL);
        let instructions = Paragraph::new(
            self.instructions
                .iter()
                .map(|i| Spans::from(i.to_string()))
                .collect::<Vec<_>>(),
        )
        .block(instructions_block)
        .scroll((self.instructions_scroll, 0));
        f.render_widget(instructions, sidebar_chunks[1]);

        // Render the simulation
        let simulation_renderer = |ctx: &mut tui::widgets::canvas::Context| {
            // Draw all the locations visited by the tail
            let tail_visited_positions = self
                .tail_visited_positions
                .iter()
                .map(|pos| (pos.x as f64, pos.y as f64))
                .collect::<Vec<_>>();
            ctx.draw(&Points {
                coords: &tail_visited_positions,
                color: Color::Rgb(100, 0, 0),
            });

            // Draw origin
            ctx.layer();
            ctx.draw(&Points {
                coords: &[(0.0, 0.0)],
                color: Color::White,
            });

            // Draw the rope itself
            ctx.layer();
            ctx.draw(&Line {
                x1: self.tail.x as _,
                y1: self.tail.y as _,
                x2: self.head.x as _,
                y2: self.head.y as _,
                color: Color::Yellow,
            });

            // Draw the rope's tail
            ctx.layer();
            ctx.draw(&Points {
                coords: &[(self.tail.x as f64, self.tail.y as f64)],
                color: Color::LightRed,
            });

            // Draw the rope's head
            ctx.layer();
            ctx.draw(&Points {
                coords: &[(self.head.x as f64, self.head.y as f64)],
                color: Color::Green,
            });
        };

        let simulation_block = Block::default().title("Simulation").borders(Borders::ALL);
        let simulation_canvas = Canvas::default()
            .block(simulation_block)
            .x_bounds([-250.0, 250.0])
            .y_bounds([-250.0, 250.0])
            .paint(simulation_renderer);
        f.render_widget(simulation_canvas, chunks[1]);

        // Render a zoomed-in view of the rope
        let closeup_block = Block::default().title("Close-up").borders(Borders::ALL);
        let closeup_canvas = Canvas::default()
            .block(closeup_block)
            .x_bounds([self.head.x as f64 - 10.0, self.head.x as f64 + 10.0])
            .y_bounds([self.head.y as f64 - 10.0, self.head.y as f64 + 10.0])
            .paint(simulation_renderer);
        f.render_widget(closeup_canvas, sidebar_chunks[2]);
    }

    fn scroll_up(&mut self, offset: u16) {
        self.instructions_scroll = self.instructions_scroll.saturating_sub(offset);
    }

    fn scroll_down(&mut self, offset: u16) {
        self.instructions_scroll = (self.instructions_scroll.saturating_add(offset))
            .min(self.instructions.len().saturating_sub(1) as _);
    }

    /// Update the app's simulation
    fn on_tick(&mut self) {
        let Some(instruction) = self.instructions.front_mut() else { return; };
        self.head += instruction.dir.delta();

        let diff = self.head - self.tail;
        let (dx, dy) = match (diff.x, diff.y) {
            // overlapping
            (0, 0) => (0, 0),

            // touching up/left/down/right
            (0, 1) | (1, 0) | (0, -1) | (-1, 0) => (0, 0),
            // touching diagonally
            (1, 1) | (1, -1) | (-1, 1) | (-1, -1) => (0, 0),

            // Need to move tail up/down/left/right
            (0, 2) => (0, 1),
            (0, -2) => (0, -1),
            (2, 0) => (1, 0),
            (-2, 0) => (-1, 0),

            // Need to move the tail diagonally right
            (2, 1) => (1, 1),
            (2, -1) => (1, -1),

            // Need to move the tail diagonally left
            (-2, 1) => (-1, 1),
            (-2, -1) => (-1, -1),

            // Need to move the tail up/down diagonally
            (1, 2) => (1, 1),
            (-1, 2) => (-1, 1),
            (1, -2) => (1, -1),
            (-1, -2) => (-1, -1),

            _ => panic!("unhandled case: tail - head = {diff:?}"),
        };

        self.tail.x += dx;
        self.tail.y += dy;
        self.tail_visited_positions.insert(self.tail);

        instruction.dist -= 1;
        if instruction.dist == 0 {
            self.instructions.pop_front();
        }
    }
}

struct AppB {
    instructions: VecDeque<Instruction>,
    knots: [GridPos; 10],
    tail_visited_positions: HashSet<GridPos>,
    instructions_scroll: u16,
}

impl AppB {
    fn new(input: &mut dyn BufRead) -> color_eyre::Result<Self> {
        let instructions = input
            .lines()
            .map(|l| -> color_eyre::Result<Instruction> {
                l.wrap_err("Could not read line from input file")
                    .map(|l| all_consuming(Instruction::parse)(&l).finish().unwrap().1)
            })
            .collect::<Result<VecDeque<Instruction>, _>>()
            .wrap_err("Could not parse instructions")?;

        Ok(Self {
            instructions,
            knots: [GridPos { x: 0, y: 0 }; 10],
            tail_visited_positions: HashSet::default(),
            instructions_scroll: 0,
        })
    }

    /// Run the app.
    fn run<B: Backend>(
        mut self,
        terminal: &mut Terminal<B>,
        tick_rate: Duration,
    ) -> color_eyre::Result<()> {
        let mut last_tick = Instant::now();
        loop {
            terminal
                .draw(|f| self.ui(f))
                .wrap_err("Error while drawing UI frame.")?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).wrap_err("Could not poll terminal for new I/O events")? {
                match event::read().wrap_err("Could not read terminal I/O event")? {
                    Event::Key(key) => match key {
                        KeyEvent {
                            code: KeyCode::Char('q'),
                            ..
                        } => return Ok(()),

                        KeyEvent {
                            code: KeyCode::Up,
                            kind: KeyEventKind::Press | KeyEventKind::Repeat,
                            ..
                        } => {
                            self.scroll_up(1);
                        }

                        KeyEvent {
                            code: KeyCode::Down,
                            kind: KeyEventKind::Press | KeyEventKind::Repeat,
                            ..
                        } => {
                            self.scroll_down(1);
                        }

                        KeyEvent {
                            code: KeyCode::PageUp,
                            kind: KeyEventKind::Press | KeyEventKind::Repeat,
                            ..
                        } => {
                            self.scroll_up(10);
                        }

                        KeyEvent {
                            code: KeyCode::PageDown,
                            kind: KeyEventKind::Press | KeyEventKind::Repeat,
                            ..
                        } => {
                            self.scroll_down(10);
                        }

                        _ => {}
                    },

                    Event::Mouse(ev) => match ev {
                        MouseEvent {
                            kind: MouseEventKind::ScrollUp,
                            ..
                        } => {
                            self.scroll_up(2);
                        }

                        MouseEvent {
                            kind: MouseEventKind::ScrollDown,
                            ..
                        } => {
                            self.scroll_down(2);
                        }

                        _ => {}
                    },

                    _ => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }
        }
    }

    /// Render the app UI to a tui frame
    fn ui<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(f.size());

        let sidebar_chunks = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Percentage(30),
            ])
            .split(chunks[0]);

        // Render the current count of visited places
        let visited_block = Block::default()
            .title("Tail locations")
            .borders(Borders::ALL);
        let visited = Paragraph::new(Span::styled(
            self.tail_visited_positions.len().to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .block(visited_block);
        f.render_widget(visited, sidebar_chunks[0]);

        // Render out all the instructions
        let instructions_block = Block::default().title("Instructions").borders(Borders::ALL);
        let instructions = Paragraph::new(
            self.instructions
                .iter()
                .map(|i| Spans::from(i.to_string()))
                .collect::<Vec<_>>(),
        )
        .block(instructions_block)
        .scroll((self.instructions_scroll, 0));
        f.render_widget(instructions, sidebar_chunks[1]);

        // Render the simulation
        let simulation_renderer = |ctx: &mut tui::widgets::canvas::Context| {
            // Draw all the locations visited by the tail
            let tail_visited_positions = self
                .tail_visited_positions
                .iter()
                .map(|pos| (pos.x as f64, pos.y as f64))
                .collect::<Vec<_>>();
            ctx.draw(&Points {
                coords: &tail_visited_positions,
                color: Color::Rgb(100, 0, 0),
            });

            // Draw origin
            ctx.layer();
            ctx.draw(&Points {
                coords: &[(0.0, 0.0)],
                color: Color::White,
            });

            // Draw the rope itself
            ctx.layer();
            for (p1, p2) in self.knots.iter().tuple_windows() {
                ctx.draw(&Line {
                    x1: p1.x as _,
                    y1: p1.y as _,
                    x2: p2.x as _,
                    y2: p2.y as _,
                    color: Color::Yellow,
                });
            }

            // Draw the rope's knots
            ctx.layer();
            let knot_points = self
                .knots
                .iter()
                .skip(1)
                .map(|pos| (pos.x as f64, pos.y as f64))
                .collect::<Vec<_>>();
            ctx.draw(&Points {
                coords: &knot_points,
                color: Color::LightRed,
            });

            // Draw the rope's head
            ctx.layer();
            ctx.draw(&Points {
                coords: &[(self.knots[0].x as f64, self.knots[0].y as f64)],
                color: Color::Green,
            });
        };

        let simulation_block = Block::default().title("Simulation").borders(Borders::ALL);
        let simulation_canvas = Canvas::default()
            .block(simulation_block)
            .x_bounds([-500.0, 500.0])
            .y_bounds([-500.0, 500.0])
            .paint(simulation_renderer);
        f.render_widget(simulation_canvas, chunks[1]);

        // Render a zoomed-in view of the rope
        let closeup_block = Block::default().title("Close-up").borders(Borders::ALL);
        let closeup_canvas = Canvas::default()
            .block(closeup_block)
            .x_bounds([self.knots[0].x as f64 - 10.0, self.knots[0].x as f64 + 10.0])
            .y_bounds([self.knots[0].y as f64 - 10.0, self.knots[0].y as f64 + 10.0])
            .paint(simulation_renderer);
        f.render_widget(closeup_canvas, sidebar_chunks[2]);
    }

    fn scroll_up(&mut self, offset: u16) {
        self.instructions_scroll = self.instructions_scroll.saturating_sub(offset);
    }

    fn scroll_down(&mut self, offset: u16) {
        self.instructions_scroll = (self.instructions_scroll.saturating_add(offset))
            .min(self.instructions.len().saturating_sub(1) as _);
    }

    /// Update the app's simulation
    fn on_tick(&mut self) {
        let Some(instruction) = self.instructions.front_mut() else { return; };
        self.knots[0] += instruction.dir.delta();

        for i in 1..self.knots.len() {
            let diff = self.knots[i - 1] - self.knots[i];
            let (dx, dy) = match (diff.x, diff.y) {
                // overlapping
                (0, 0) => (0, 0),

                // touching up/left/down/right
                (0, 1) | (1, 0) | (0, -1) | (-1, 0) => (0, 0),
                // touching diagonally
                (1, 1) | (1, -1) | (-1, 1) | (-1, -1) => (0, 0),

                // Need to move knot up/down/left/right
                (0, 2) => (0, 1),
                (0, -2) => (0, -1),
                (2, 0) => (1, 0),
                (-2, 0) => (-1, 0),

                // Need to move the knot diagonally right
                (2, 1) => (1, 1),
                (2, -1) => (1, -1),

                // Need to move the knot diagonally left
                (-2, 1) => (-1, 1),
                (-2, -1) => (-1, -1),

                // Need to move the knot up/down diagonally
                (1, 2) => (1, 1),
                (-1, 2) => (-1, 1),
                (1, -2) => (1, -1),
                (-1, -2) => (-1, -1),

                // Need to move the knot diagonally
                (-2, -2) => (-1, -1),
                (-2, 2) => (-1, 1),
                (2, -2) => (1, -1),
                (2, 2) => (1, 1),

                _ => panic!("unhandled case: knots[{}] - knots[{i}] = {diff:?}", i - 1),
            };

            self.knots[i].x += dx;
            self.knots[i].y += dy;

            if i == self.knots.len() - 1 {
                self.tail_visited_positions.insert(self.knots[i]);
            }
        }

        instruction.dist -= 1;
        if instruction.dist == 0 {
            self.instructions.pop_front();
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GridPos {
    x: i32,
    y: i32,
}

impl fmt::Debug for GridPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("").field(&self.x).field(&self.y).finish()
    }
}

impl ops::Add for GridPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::AddAssign for GridPos {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Sub for GridPos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::SubAssign for GridPos {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Try to parse a string into a direction.
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            value(Self::Up, tag("U")),
            value(Self::Down, tag("D")),
            value(Self::Left, tag("L")),
            value(Self::Right, tag("R")),
        ))(i)
    }

    /// Turn a direction into a "unit vector", represented by a 2D grid position
    ///
    /// The world coordinate system is orientated so that positive x is rightwards
    /// and positive y is upwards, like so:
    ///
    /// ```text
    ///            (+y)
    ///
    ///             ↑
    ///             |
    ///    (-x) ----+---→ (+x)
    ///             |
    ///             |
    ///
    ///            (-y)
    /// ```
    fn delta(self) -> GridPos {
        match self {
            Self::Up => GridPos { x: 0, y: 1 },
            Self::Down => GridPos { x: 0, y: -1 },
            Self::Left => GridPos { x: -1, y: 0 },
            Self::Right => GridPos { x: 1, y: 0 },
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Up => write!(f, "↑"),
            Direction::Down => write!(f, "↓"),
            Direction::Right => write!(f, "→"),
            Direction::Left => write!(f, "←"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Instruction {
    dir: Direction,
    dist: u32,
}

impl Instruction {
    /// Try to parse a direction and a distance into a movement instruction.
    fn parse(i: &str) -> IResult<&str, Self> {
        map(
            tuple((
                Direction::parse,
                preceded(space1, nom::character::complete::u32),
            )),
            |(dir, dist)| Self { dir, dist },
        )(i)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let arrow = self.dir.to_string();
        let arrows = arrow.repeat(self.dist as _);
        arrows.fmt(f)
    }
}
