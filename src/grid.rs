//! Generalized utilities for working with grids.

use std::fmt;

/// A 2D grid coordinate, where `x` and `y` are represented as `usize`s.
///
/// Can be used for referencing cells in a [`Grid`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GridCoord {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl fmt::Debug for GridCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("").field(&self.x).field(&self.y).finish()
    }
}

impl From<(usize, usize)> for GridCoord {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

/// A 2D grid of arbitrary values with a constant width and height.
pub(crate) struct Grid<T> {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) data: Vec<T>,
}

impl<T> Grid<T>
where
    T: Default + Clone,
{
    /// Create a new grid with a constant width and height.
    ///
    /// The grid will be filled with default-initialized clones of whatever type `T` is.
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![T::default(); width * height],
        }
    }
}

impl<T> Grid<T> {
    pub(crate) const fn in_bounds(&self, coord: GridCoord) -> bool {
        coord.x < self.width && coord.y < self.height
    }

    /// Get a _mutable_ reference to a value at some grid coordinate.
    ///
    /// Returns `None` if `coord` is out-of-bounds.
    pub(crate) fn cell_mut(&mut self, coord: GridCoord) -> Option<&mut T> {
        if !self.in_bounds(coord) {
            return None;
        }
        Some(&mut self.data[coord.y * self.width + coord.x])
    }

    /// Get a reference to a value at some grid coordinate.
    ///
    /// Returns `None` if `coord` is out-of-bounds.
    pub(crate) fn cell(&self, coord: GridCoord) -> Option<&T> {
        if !self.in_bounds(coord) {
            return None;
        }
        Some(&self.data[coord.y * self.width + coord.x])
    }

    /// Get the grid's constant width.
    #[inline]
    pub(crate) const fn width(&self) -> usize {
        self.width
    }

    /// Get the grid's constant height.
    #[inline]
    pub(crate) const fn height(&self) -> usize {
        self.height
    }

    pub(crate) const fn num_cells(&self) -> usize {
        self.width * self.height
    }
}

impl<T> fmt::Debug for Grid<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            writeln!(f, "{}x{} grid:", self.width, self.height)?;
            for y in 0..self.height {
                for x in 0..self.width {
                    let cell = self.cell((x, y).into()).unwrap();
                    cell.fmt(f)?;
                }
                writeln!(f)?;
            }
        } else {
            f.debug_struct(&format!("Grid<{}>", std::any::type_name::<T>()))
                .field("width", &self.width)
                .field("height", &self.height)
                .finish_non_exhaustive()?;
        }

        Ok(())
    }
}
