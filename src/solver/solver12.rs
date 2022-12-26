use std::{
    collections::{HashMap, HashSet},
    fmt,
    fs::File,
    io::{self, BufReader, Read},
    time::{Duration, Instant},
};

use color_eyre::eyre::Context;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        canvas::{self, Canvas},
        Block, Borders, Paragraph,
    },
    Frame, Terminal,
};

use crate::grid::{Grid, GridCoord};

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver12;

impl ChallengeSolver for Solver12 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        12
    }

    fn solve_a(&mut self, mut input: BufReader<File>) -> color_eyre::Result<()> {
        // parse grid
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;
        let grid = Grid::parse(&input_buf);

        // Initialize app
        let app = App::new(grid, InitialSet::StartingCell);

        // setup terminal
        enable_raw_mode().wrap_err("Could not initialize terminal UI")?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .wrap_err("Could not initialize terminal UI")?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).wrap_err("Could not initialize terminal UI")?;

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

        Ok(())
    }

    fn solve_b(&mut self, mut input: BufReader<File>) -> color_eyre::Result<()> {
        // parse grid
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;
        let grid = Grid::parse(&input_buf);

        // Initialize app
        let app = App::new(grid, InitialSet::LowestElevationCell);

        // setup terminal
        enable_raw_mode().wrap_err("Could not initialize terminal UI")?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .wrap_err("Could not initialize terminal UI")?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).wrap_err("Could not initialize terminal UI")?;

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

        Ok(())
    }
}

enum InitialSet {
    StartingCell,
    LowestElevationCell,
}

struct App {
    grid: Grid<Cell>,
    visited: HashMap<GridCoord, CellRecord>,
    current: HashSet<GridCoord>,
    num_steps: usize,
    end_found: bool,

    initial_set: InitialSet,

    show_glyphs: bool,
    show_walkable_neighbors: bool,
}

impl App {
    fn new(grid: Grid<Cell>, initial_set: InitialSet) -> Self {
        Self {
            grid,
            visited: Default::default(),
            current: Default::default(),
            num_steps: 0,
            end_found: false,

            initial_set,

            show_glyphs: false,
            show_walkable_neighbors: false,
        }
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
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }) => return Ok(()),

                    Event::Key(KeyEvent {
                        code: KeyCode::Char('g'),
                        kind: KeyEventKind::Press | KeyEventKind::Repeat,
                        ..
                    }) => {
                        self.show_glyphs = !self.show_glyphs;
                    }

                    Event::Key(KeyEvent {
                        code: KeyCode::Char('n'),
                        kind: KeyEventKind::Press | KeyEventKind::Release,
                        ..
                    }) => {
                        self.show_walkable_neighbors = !self.show_walkable_neighbors;
                    }

                    _ => (),
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
        // Split screen up into main areas
        let chunks = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(f.size());
        let main_chunk = &chunks[0];
        let info_chunk = &chunks[1];

        // Render the main simulation
        let main_block = Block::default().borders(Borders::NONE);
        let main_canvas = Canvas::default()
            .block(main_block)
            .x_bounds([0.0, self.grid.width() as f64])
            .y_bounds([0.0, self.grid.height() as f64])
            .background_color(Color::Rgb(0, 0, 0))
            .paint(|ctx| {
                // Paint the grid
                let grid_height = self.grid.height();
                let grid_width = self.grid.width();
                for y in 0..grid_height {
                    for x in 0..grid_width {
                        let cell = self.grid.cell((x, grid_height - 1 - y).into()).unwrap();

                        let (glyph, color) = match cell {
                            Cell::Start => ("S".to_string(), Color::Rgb(216, 27, 96)),
                            Cell::End => ("E".to_string(), Color::Rgb(30, 136, 229)),
                            Cell::Square(elevation) => {
                                let glyph = format!("{elevation}");
                                let elevation = *elevation as f32 / 25.0;
                                let f = (elevation * 255.0) as u8;
                                (glyph, Color::Rgb(f, f, f))
                            }
                        };
                        let Color::Rgb(r, g, b) = color else { unreachable!(); };

                        let fill_points = (0..=20)
                            .flat_map(|fill_x| {
                                let fill_x = fill_x as f64 / 20.0 + x as f64;
                                (0..=20).map(move |fill_y| {
                                    let fill_y = fill_y as f64 / 20.0 + y as f64;
                                    (fill_x, fill_y)
                                })
                            })
                            .collect::<Vec<_>>();

                        ctx.draw(&canvas::Points {
                            coords: &fill_points,
                            color,
                        });

                        if self.show_glyphs {
                            ctx.print(
                                x as f64 + 0.5,
                                y as f64 + 0.5,
                                Spans(vec![Span::styled(
                                    glyph,
                                    Style::default().bg(color).fg(Color::Rgb(
                                        255 - r,
                                        255 - g,
                                        255 - b,
                                    )),
                                )]),
                            );
                        }
                    }
                }

                // Optionally paint walkable neighbors
                if self.show_walkable_neighbors {
                    ctx.layer();

                    for y in 0..grid_height {
                        for x in 0..grid_width {
                            let coord: GridCoord = (x, grid_height - 1 - y).into();
                            for ncoord in self.grid.walkable_neighbors(coord) {
                                let (x, y) = (x as f64, y as f64);
                                let dx = ncoord.x as f64 - x;
                                let dy = grid_height as f64 - 1.0 - ncoord.y as f64 - y;

                                ctx.draw(&canvas::Line {
                                    x1: x + 0.5 + dx * 0.05,
                                    y1: y + 0.5 + dy * 0.05,
                                    x2: x + 0.5 + dx * 0.45,
                                    y2: y + 0.5 + dy * 0.45,
                                    color: Color::Rgb(255, 193, 7),
                                });
                                ctx.draw(&canvas::Rectangle {
                                    x: x + 0.5 + dx * 0.45 - 0.05,
                                    y: y + 0.5 + dy * 0.45 - 0.05,
                                    width: 0.1,
                                    height: 0.1,
                                    color: Color::Rgb(255, 193, 7),
                                })
                            }
                        }
                    }
                }

                // Render the search lines
                ctx.layer();
                for coord in self.current.iter() {
                    // use a text label as a "circle"
                    ctx.print(
                        coord.x as f64 + 0.5,
                        grid_height as f64 - (coord.y as f64 + 0.5),
                        Spans(vec![Span::styled(
                            "●",
                            Style::default().fg(Color::Rgb(255, 193, 7)),
                        )]),
                    );

                    // draw a polyline from the current coord all the way back to the start
                    let record = self.visited.get(coord).unwrap();
                    let mut curr = record;
                    let mut coord = *coord;
                    while let Some(prev) = curr.prev.as_ref() {
                        curr = self.visited.get(prev).unwrap();

                        let (x, y) = (prev.x as f64, prev.y as f64);
                        let dx = coord.x as f64 - x;
                        let dy = coord.y as f64 - y;

                        ctx.draw(&canvas::Line {
                            x1: x + 0.5 + dx * 0.2,
                            y1: grid_height as f64 - (y + 0.5 + dy * 0.2),
                            x2: x + 0.5 + dx * 0.8,
                            y2: grid_height as f64 - (y + 0.5 + dy * 0.8),
                            color: Color::Rgb(255, 193, 7),
                        });

                        coord = *prev;
                    }
                }
            });
        f.render_widget(main_canvas, *main_chunk);

        // Split the information block up into areas
        let info_chunks = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(27),
                Constraint::Length(28),
            ])
            .split(*info_chunk);
        let info_main_chunk = info_chunks[0];
        let info_glyph_display_chunk = info_chunks[1];
        let info_walkable_neighbors_chunk = info_chunks[2];

        // Render simulation information
        let info_block = Block::default().borders(Borders::ALL);
        let info_paragraph = Paragraph::new(self.status_text()).block(info_block);
        f.render_widget(info_paragraph, info_main_chunk);

        // Render instructions on how to show the debug elevation glyphs
        let info_glyph_display_block = Block::default().borders(Borders::ALL);
        let info_glyph_display_paragraph = Paragraph::new(Spans(vec![
            Span::raw("Elevation ["),
            Span::styled(
                "g",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            ),
            Span::raw("]lyph display"),
        ]))
        .block(info_glyph_display_block);
        f.render_widget(info_glyph_display_paragraph, info_glyph_display_chunk);

        // Render instructions on how to show the walkable neigbours
        let info_walkable_neighbors_block = Block::default().borders(Borders::ALL);
        let info_walkable_neighbors_paragraph = Paragraph::new(Spans(vec![
            Span::raw("Show walkable ["),
            Span::styled(
                "n",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            ),
            Span::raw("]eighbours"),
        ]))
        .block(info_walkable_neighbors_block);
        f.render_widget(
            info_walkable_neighbors_paragraph,
            info_walkable_neighbors_chunk,
        )
    }

    /// Update the app's simulation
    fn on_tick(&mut self) {
        if self.end_found {
            return;
        }

        let grid_height = self.grid.height();
        let grid_width = self.grid.width();

        if self.current.is_empty() {
            // find start coordinate
            match self.initial_set {
                InitialSet::StartingCell => {
                    'outer: for y in 0..grid_height {
                        for x in 0..grid_width {
                            let coord = (x, y).into();
                            if let Cell::Start = self.grid.cell(coord).unwrap() {
                                self.current.insert(coord);
                                self.visited.insert(coord, CellRecord { prev: None });
                                break 'outer;
                            }
                        }
                    }
                }

                InitialSet::LowestElevationCell => {
                    for y in 0..grid_height {
                        for x in 0..grid_width {
                            let coord = (x, y).into();
                            if let Cell::Start | Cell::Square(0) = self.grid.cell(coord).unwrap() {
                                self.current.insert(coord);
                                self.visited.insert(coord, CellRecord { prev: None });
                            }
                        }
                    }
                }
            }
        } else {
            // Visit the current cells' neigbours
            let current = std::mem::take(&mut self.current);
            let mut next = HashSet::new();
            let mut visited = std::mem::take(&mut self.visited);

            'outer: for curr in current {
                for ncoord in self.grid.walkable_neighbors(curr) {
                    if visited.contains_key(&ncoord) {
                        // don't visit it again!
                        continue;
                    }

                    if !self.end_found {
                        if let Some(&Cell::End) = self.grid.cell(ncoord) {
                            // found the end coordinate!
                            self.end_found = true;
                            break 'outer;
                        }
                    }

                    visited.insert(ncoord, CellRecord { prev: Some(curr) });
                    next.insert(ncoord);
                }
            }

            self.current = next;
            self.visited = visited;
            self.num_steps += 1;
        }
    }

    fn num_visited(&self) -> usize {
        self.visited.len()
    }

    const fn num_steps(&self) -> usize {
        self.num_steps
    }

    fn status_text(&self) -> Spans {
        let percent = self.num_visited() as f64 / self.grid.num_cells() as f64 * 100.0;
        let mut spans = vec![Span::raw(format!(
            "{} steps, {}/{} visited ({percent:.01}%) - ",
            self.num_steps(),
            self.num_visited(),
            self.grid.num_cells()
        ))];

        if self.end_found {
            spans.push(Span::styled(
                "COMPLETE",
                Style::default()
                    .fg(Color::Rgb(193, 255, 7))
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                "SEARCHING",
                Style::default().fg(Color::Rgb(255, 193, 7)),
            ));
        }

        Spans(spans)
    }
}

#[derive(Clone, Copy)]
enum Cell {
    /// Start position
    Start,
    /// End position
    End,
    /// Square with given elevation
    Square(u8),
}

impl Cell {
    fn elevation(&self) -> u8 {
        match self {
            Self::Start => 0,
            Self::End => 25,
            Self::Square(e) => *e,
        }
    }
}

impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start => write!(f, "S"),
            Self::End => write!(f, "E"),
            Self::Square(elevation) => write!(f, "{}", (b'a' + elevation) as char),
        }
    }
}

struct CellRecord {
    prev: Option<GridCoord>,
}

trait GridExt {
    /// Parse the input file into a heightmap grid
    fn parse(input: &str) -> Self;

    /// Get the walkable neighbours next to a grid cell.
    fn walkable_neighbors(&self, coord: GridCoord) -> Box<dyn Iterator<Item = GridCoord> + '_>;
}

impl GridExt for Grid<Cell> {
    fn parse(input: &str) -> Self {
        let first_line = input.lines().next().unwrap();
        let width = first_line.len();
        let height = input.lines().count();

        let data = input
            .chars()
            .filter_map(|c| match c {
                'S' => Some(Cell::Start),
                'E' => Some(Cell::End),
                'a'..='z' => Some(Cell::Square(c as u8 - b'a')),
                '\r' | '\n' => None,
                _ => panic!("invalid character: {c}"),
            })
            .collect::<Vec<_>>();

        Self {
            width,
            height,
            data,
        }
    }

    fn walkable_neighbors(&self, coord: GridCoord) -> Box<dyn Iterator<Item = GridCoord> + '_> {
        let curr_elev = self.cell(coord).unwrap().elevation();
        let deltas: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        Box::new(deltas.into_iter().filter_map(move |(dx, dy)| {
            Some(GridCoord {
                x: coord.x.checked_add_signed(dx)?,
                y: coord.y.checked_add_signed(dy)?,
            })
            .filter(|&coord| self.in_bounds(coord))
            .filter(|&coord| {
                let other_elev = self.cell(coord).unwrap().elevation();
                other_elev <= curr_elev + 1
            })
        }))
    }
}
