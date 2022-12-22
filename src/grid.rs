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
    width: usize,
    height: usize,
    data: Vec<T>,
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
    const fn in_bounds(&self, coord: GridCoord) -> bool {
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
}
