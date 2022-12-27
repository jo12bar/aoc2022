use std::{
    fmt,
    fs::File,
    io::{BufReader, Read},
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, MutexGuard,
    },
    time::{Duration, Instant},
};

use color_eyre::eyre::Context;
use eframe::emath;
use genawaiter::rc::Gen;
use miette::GraphicalReportHandler;
use nom::{
    character::complete::{self as nom_cc, space0},
    combinator::map,
    error::ParseError,
    multi::separated_list1,
    sequence::{separated_pair, tuple},
    IResult,
};
use nom_locate::LocatedSpan;
use nom_supreme::{
    error::{BaseErrorKind, ErrorTree, GenericErrorTree},
    final_parser::final_parser,
    tag::{complete::tag, TagError},
};
use once_cell::sync::OnceCell;

use crate::atomic::AtomicF32;

use super::ChallengeSolver;

#[derive(Debug, Default)]
pub struct Solver14;

impl ChallengeSolver for Solver14 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        14
    }

    fn solve_a(&mut self, mut input: BufReader<File>) -> color_eyre::Result<()> {
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;

        let input = Span::new(&input_buf);

        let polylines_res: Result<_, ErrorTree<Span>> =
            final_parser(Polyline::parse_all::<ErrorTree<Span>>)(input);

        let mut polylines = match polylines_res {
            Ok(polylines) => polylines,

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

        // Setup the simulation grid
        let grid = Grid::new(&mut polylines, false);

        // Start the eframe app
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "AOC2022 C14A",
            native_options,
            Box::new(move |cc| Box::new(App::new(cc, grid))),
        );

        Ok(())
    }

    fn solve_b(&mut self, mut input: BufReader<File>) -> color_eyre::Result<()> {
        let mut input_buf = String::new();
        input
            .read_to_string(&mut input_buf)
            .wrap_err("Could not read input file to string")?;

        let input = Span::new(&input_buf);

        let polylines_res: Result<_, ErrorTree<Span>> =
            final_parser(Polyline::parse_all::<ErrorTree<Span>>)(input);

        let mut polylines = match polylines_res {
            Ok(polylines) => polylines,

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

        // Setup the simulation grid
        let grid = Grid::new(&mut polylines, true);

        // Start the eframe app
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "AOC2022 C14B",
            native_options,
            Box::new(move |cc| Box::new(App::new(cc, grid))),
        );

        Ok(())
    }
}

/// The main eframe app
struct App {
    grid: Arc<Grid>,
    speed_factor: Arc<AtomicF32>,
    simulation_running: Arc<AtomicBool>,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>, grid: Arc<Grid>) -> Self {
        Self {
            grid,
            speed_factor: Arc::new(AtomicF32::new(1.0)),
            simulation_running: Arc::new(AtomicBool::new(false)),
        }
    }

    fn start_simulation(&self, ctx: egui::Context) -> Option<std::thread::JoinHandle<usize>> {
        if self.simulation_running.load(Ordering::SeqCst) {
            None
        } else {
            self.simulation_running.store(true, Ordering::SeqCst);

            let base_rate = Duration::from_secs_f64(1.0 / 30.0);

            let grid = Arc::clone(&self.grid);
            let speed_factor = Arc::clone(&self.speed_factor);
            let simulation_running = Arc::clone(&self.simulation_running);

            grid.reset();

            Some(std::thread::spawn(move || {
                println!("Starting simulation");

                let mut last_tick = Instant::now();

                while simulation_running.load(Ordering::Relaxed) {
                    let res = grid.step();
                    ctx.request_repaint();

                    if res {
                        break;
                    }

                    let speed_factor = speed_factor.load(Ordering::Relaxed);

                    if speed_factor <= f32::EPSILON {
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }

                    let elapsed = last_tick.elapsed();
                    let time_left_over =
                        Duration::from_secs_f32(base_rate.as_secs_f32() / speed_factor)
                            .checked_sub(elapsed);
                    if let Some(t) = time_left_over {
                        std::thread::sleep(t);
                    }
                    last_tick = Instant::now();
                }

                println!("Stopping simulation");

                grid.settled.load(Ordering::Relaxed)
            }))
        }
    }

    fn ui_controls(&mut self, ui: &mut egui::Ui, ctx: egui::Context) -> egui::Response {
        ui.horizontal(|ui| {
            let mut local_speed_factor = self.speed_factor.load(Ordering::Acquire);
            ui.add(
                egui::Slider::new(&mut local_speed_factor, 0.0..=32.0)
                    .text("Speed")
                    .suffix("x")
                    .logarithmic(true)
                    .smallest_positive(0.1),
            );
            self.speed_factor
                .store(local_speed_factor, Ordering::Release);

            ui.separator();
            let simulation_running = self.simulation_running.load(Ordering::Relaxed);
            if ui
                .add_enabled(!simulation_running, egui::Button::new("▶"))
                .clicked()
            {
                self.start_simulation(ctx);
            }

            if ui
                .add_enabled(simulation_running, egui::Button::new("■"))
                .clicked()
            {
                self.simulation_running.store(false, Ordering::Relaxed);
            }

            ui.separator();
            ui.label(format!(
                "Settled grains: {}",
                self.grid.settled.load(Ordering::Relaxed)
            ));
        })
        .response
    }

    fn ui_canvas(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let grid_origin = *self.grid.origin.get().unwrap();

        let (response, painter) = ui.allocate_painter(
            ui.available_size_before_wrap(),
            egui::Sense::focusable_noninteractive(),
        );

        let to_screen = emath::RectTransform::from_to(
            egui::Rect::from_center_size(
                (
                    grid_origin.x as f32 + self.grid.width() as f32 / 2.0,
                    grid_origin.y as f32 + self.grid.height() as f32 / 2.0,
                )
                    .into(),
                (
                    self.grid.width() as f32 * 1.1,
                    self.grid.height() as f32 * 1.1,
                )
                    .into(),
            ),
            egui::Rect::from_center_size(
                response.rect.center(),
                emath::vec2(
                    self.grid.aspect_ratio() * response.rect.height(),
                    response.rect.height(),
                ),
            ),
        );

        for y in 0..self.grid.height() {
            for x in 0..self.grid.width() {
                let point = Point {
                    x: x as _,
                    y: y as _,
                } + grid_origin;

                let cell = self.grid.cell(point).unwrap();
                let color = match cell {
                    // don't actually draw air cells
                    Cell::Air => {
                        continue;
                    }

                    Cell::Rock => egui::Color32::from_rgb(165, 156, 145),
                    Cell::Sand => egui::Color32::from_rgb(206, 201, 139),
                };

                painter.rect_filled(
                    to_screen.transform_rect(egui::Rect::from_min_size(
                        emath::pos2(point.x as _, point.y as _),
                        emath::vec2(1.0, 1.0),
                    )),
                    0.0,
                    color,
                );
            }
        }

        {
            let current_grains = self.grid.current_grains.lock().unwrap();
            for point in current_grains.iter() {
                painter.rect_filled(
                    to_screen.transform_rect(egui::Rect::from_min_size(
                        emath::pos2(point.x as _, point.y as _),
                        emath::vec2(1.0, 1.0),
                    )),
                    4.0,
                    egui::Color32::from_rgb(255, 193, 7),
                );
            }
        }

        response
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui_controls(ui, ctx.clone());
            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                self.ui_canvas(ui);
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.simulation_running.store(false, Ordering::SeqCst);
    }
}

/// Sand spawns at point (500, 0)
const SAND_SPAWN: Point = Point { x: 500, y: 0 };

type Span<'a> = LocatedSpan<&'a str>;

#[derive(
    Copy, Clone, PartialEq, Eq, derive_more::Add, derive_more::AddAssign, derive_more::Sub,
)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    /// Try to parse a point from a string like `-43,2`.
    fn parse<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + TagError<Span<'a>, &'static str>,
    {
        let (i, (x, y)) = separated_pair(nom_cc::i32, tag(","), nom_cc::i32)(i)?;

        Ok((i, Self { x, y }))
    }

    /// For each component (x, y) of self, returns a number representing the sign.
    ///
    /// - 0 if the number is zero
    /// - 1 if the number is positive
    /// - -1 if the number is negative
    fn signum(self) -> Self {
        Self {
            x: self.x.signum(),
            y: self.y.signum(),
        }
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("").field(&self.x).field(&self.y).finish()
    }
}

#[derive(Debug, Clone)]
struct Polyline {
    points: Vec<Point>,
}

impl Polyline {
    /// Try to parse a polyline from a list like `3,2 -> 6,2 -> 6,-3`
    fn parse<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + TagError<Span<'a>, &'static str>,
    {
        map(
            separated_list1(tuple((space0, tag("->"), space0)), Point::parse),
            |points| Self { points },
        )(i)
    }

    /// Parse all of the challenge input into a list of polylines
    fn parse_all<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Vec<Self>, E>
    where
        E: ParseError<Span<'a>> + TagError<Span<'a>, &'static str>,
    {
        nom::sequence::terminated(
            separated_list1(nom_cc::newline, Self::parse),
            nom_cc::multispace0,
        )(i)
    }

    /// Iterate over points in the polyline
    fn path_points(&self) -> impl Iterator<Item = Point> + '_ {
        Gen::new(|co| async move {
            let mut points = self.points.iter().copied();
            let Some(mut a) = points.next() else { return };
            co.yield_(a).await;

            loop {
                let Some(b) = points.next() else { return };
                let delta = (b - a).signum();
                assert!((delta.x == 0) ^ (delta.y == 0));

                loop {
                    a += delta;
                    co.yield_(a).await;
                    if a == b {
                        break;
                    }
                }
            }
        })
        .into_iter()
    }
}

#[derive(Debug, Clone, Copy)]
enum Cell {
    Air,
    Rock,
    Sand,
}

/// A world grid.
///
/// Positive x is rightwards, positive y is downwards.
struct Grid {
    origin: OnceCell<Point>,
    width: AtomicUsize,
    height: AtomicUsize,
    cells: Mutex<Vec<Cell>>,
    orig_cells: Mutex<Vec<Cell>>,
    settled: AtomicUsize,
    current_grains: Mutex<Vec<Point>>,
}

impl Grid {
    fn new(rock_walls: &mut Vec<Polyline>, with_floor: bool) -> Arc<Self> {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);

        for point in rock_walls
            .iter()
            .flat_map(|pl| pl.points.iter())
            .chain(std::iter::once(&SAND_SPAWN))
        {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        if with_floor {
            let floor_y = max_y + 2;
            min_x = 300;
            max_x = 700;
            max_y = floor_y;
            rock_walls.push(Polyline {
                points: vec![
                    Point {
                        x: min_x,
                        y: floor_y,
                    },
                    Point {
                        x: max_x,
                        y: floor_y,
                    },
                ],
            });
        }

        dbg!(min_x, max_x);
        dbg!(min_y, max_y);

        let origin = OnceCell::with_value(Point { x: min_x, y: min_y });
        let w = usize::try_from(max_x - min_x + 1).unwrap();
        let width = AtomicUsize::from(w);
        let h = usize::try_from(max_y - min_y + 1).unwrap();
        let height = AtomicUsize::from(h);

        dbg!(&origin, &width, &height);

        let mut grid = Self {
            origin,
            width,
            height,
            cells: Mutex::new(vec![Cell::Air; w * h]),
            orig_cells: Mutex::new(Vec::new()),
            settled: AtomicUsize::from(0),
            current_grains: Mutex::new(Vec::new()),
        };

        for point in rock_walls.iter().flat_map(|pl| pl.path_points()) {
            *grid.cell_mut(point).unwrap() = Cell::Rock;
        }

        let cells = grid.cells.lock().unwrap().clone();
        let orig_cells = grid.orig_cells.get_mut().unwrap();
        let _ = std::mem::replace(orig_cells, cells);

        Arc::new(grid)
    }

    fn cell_index(&self, point: Point) -> Option<usize> {
        // If coords are negative after offsetting, they're outside the grid
        let Point { x, y } = point - *self.origin.get().unwrap();

        let x: usize = x.try_into().ok()?;
        let y: usize = y.try_into().ok()?;

        let width = self.width();
        let height = self.height();

        if x < width && y < height {
            Some(y * width + x)
        } else {
            None
        }
    }

    /// Get a _mutable_ reference to a value at some grid coordinate.
    ///
    /// Returns `None` if `coord` is out-of-bounds.
    pub(crate) fn cell_mut(&mut self, point: Point) -> Option<&mut Cell> {
        let idx = self.cell_index(point)?;
        let cells = self.cells.get_mut().ok()?;
        Some(&mut cells[idx])
    }

    /// Get an _mutable_ reference to a value at some grid coordinate, **and**
    /// a [`std::sync::MutexGuard`] that must be dropped at the same time as the
    /// reference.
    ///
    /// As long as the [`std::sync::MutexGuard`] is held onto, any further calls
    /// to this function will result in blocking the tread and potential deadlocks.
    pub(crate) fn cell_mut_ref(&self, point: Point) -> Option<impl DerefMut<Target = Cell> + '_> {
        let idx = self.cell_index(point)?;
        let guard = self.cells.lock().ok()?;

        struct CellMutexGuardBundle<'a> {
            guard: MutexGuard<'a, Vec<Cell>>,
            idx: usize,
        }

        impl<'a> Deref for CellMutexGuardBundle<'a> {
            type Target = Cell;

            fn deref(&self) -> &Self::Target {
                &self.guard[self.idx]
            }
        }

        impl<'a> DerefMut for CellMutexGuardBundle<'a> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.guard[self.idx]
            }
        }

        Some(CellMutexGuardBundle { guard, idx })
    }

    /// Get a copy of a value at some grid coordinate.
    ///
    /// Returns `None` if `coord` is out-of-bounds.
    pub(crate) fn cell(&self, point: Point) -> Option<Cell> {
        let idx = self.cell_index(point)?;
        let cells = self.cells.lock().unwrap();
        Some(cells[idx])
    }

    /// Get the grid's constant width.
    #[inline]
    pub(crate) fn width(&self) -> usize {
        self.width.load(Ordering::Relaxed)
    }

    /// Get the grid's constant height.
    #[inline]
    pub(crate) fn height(&self) -> usize {
        self.height.load(Ordering::Relaxed)
    }

    /// Aspect ratio as `self.width / self.height`
    pub(crate) fn aspect_ratio(&self) -> f32 {
        self.width() as f32 / self.height() as f32
    }

    /// Reset the simulation
    fn reset(&self) {
        let orig_cells = self.orig_cells.lock().unwrap().clone();

        {
            let mut cells = self.cells.lock().unwrap();
            *cells.as_mut() = orig_cells;
        }

        self.settled.store(0, Ordering::Relaxed);

        {
            let mut current_grains = self.current_grains.lock().unwrap();
            *current_grains.as_mut() = Vec::new();
        }
    }

    /// Step the simulation.
    ///
    /// Returns `true` if the simulation has completed.
    fn step(&self) -> bool {
        if matches!(self.cell(Point { x: 500, y: 0 }).unwrap(), Cell::Sand) {
            // don't step, we're done
            return true;
        }

        let mut current_grains = {
            let mut current_grains = self.current_grains.lock().unwrap();
            std::mem::take(&mut *current_grains)
        };

        let _ = VecExt::drain_filter(&mut current_grains, |grain| {
            let straight_down = *grain + Point { x: 0, y: 1 };
            let down_left = *grain + Point { x: -1, y: 1 };
            let down_right = *grain + Point { x: 1, y: 1 };
            let options = [straight_down, down_left, down_right];

            // Can we move?
            if let Some(pos) = options
                .into_iter()
                .find(|pos| matches!(self.cell(*pos), Some(Cell::Air)))
            {
                *grain = pos;
                return false; // keep it
            }

            // If not, are we moving off-screen?
            if options.into_iter().any(|pos| self.cell(pos).is_none()) {
                return true; // remove it
            }

            // If not, then we've settled
            self.settled.fetch_add(1, Ordering::Relaxed);

            {
                let mut cell_ref = self.cell_mut_ref(*grain).unwrap();
                *cell_ref = Cell::Sand;
            }

            // Remove it
            true
        })
        .count();

        current_grains.push(SAND_SPAWN);

        {
            let mut cg = self.current_grains.lock().unwrap();
            *cg = current_grains;
        }

        false
    }
}

impl fmt::Debug for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let width = self.width();
            let height = self.height();
            writeln!(
                f,
                "{}x{} grid with origin at {:?}",
                width,
                height,
                self.origin.get().unwrap()
            )?;
            for y in 0..height {
                for x in 0..width {
                    let p = Point {
                        x: x as _,
                        y: y as _,
                    } + *self.origin.get().unwrap();
                    let cell = self.cell(p).unwrap();
                    let glyph = match cell {
                        Cell::Air => "░",
                        Cell::Rock => "█",
                        Cell::Sand => "○",
                    };
                    write!(f, "{glyph}")?;
                }
                writeln!(f)?;
            }

            Ok(())
        } else {
            f.debug_struct("Grid")
                .field("origin", &self.origin)
                .field("width", &self.width)
                .field("height", &self.height)
                .finish_non_exhaustive()
        }
    }
}

trait VecExt<T> {
    /// The [new `drain_filter` iterator][Vec::drain_filter] from the standard library, currently
    /// only available in nightly.
    fn drain_filter<F>(&mut self, filter: F) -> DrainFilter<T, F>
    where
        F: FnMut(&mut T) -> bool;
}

impl<T> VecExt<T> for Vec<T> {
    fn drain_filter<F>(&mut self, filter: F) -> DrainFilter<T, F>
    where
        F: FnMut(&mut T) -> bool,
    {
        let old_len = self.len();

        // Gaurd against us getting leaked (leak amplification)
        unsafe {
            self.set_len(0);
        }

        DrainFilter {
            vec: self,
            idx: 0,
            del: 0,
            old_len,
            pred: filter,
        }
    }
}

/// An iterator produced by calling `drain_filter` on Vec.
#[derive(Debug)]
struct DrainFilter<'a, T: 'a, F>
where
    F: FnMut(&mut T) -> bool,
{
    vec: &'a mut Vec<T>,
    idx: usize,
    del: usize,
    old_len: usize,
    pred: F,
}

impl<'a, T, F> Iterator for DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        unsafe {
            while self.idx != self.old_len {
                let i = self.idx;
                self.idx += 1;
                let v = std::slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
                if (self.pred)(&mut v[i]) {
                    self.del += 1;
                    return Some(std::ptr::read(&v[i]));
                } else if self.del > 0 {
                    v.swap(i - self.del, i);
                }
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.old_len - self.idx))
    }
}

impl<'a, T, F> Drop for DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    fn drop(&mut self) {
        for _ in self.by_ref() {}

        unsafe {
            self.vec.set_len(self.old_len - self.del);
        }
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
