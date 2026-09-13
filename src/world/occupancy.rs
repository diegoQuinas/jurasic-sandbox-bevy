use bevy::prelude::*;

use super::Position;

/// Flat 2D occupancy grid (`y * width + x`). `Position.z` is ignored.
#[derive(Resource)]
pub struct Occupancy {
    pub cells: Vec<Option<Entity>>,
    width: usize,
    #[allow(dead_code)]
    height: usize,
}

impl Occupancy {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![None; width * height],
            width,
            height,
        }
    }

    /// `None` for both an empty cell and an out-of-bounds position.
    #[inline]
    pub fn get(&self, pos: Position) -> Option<Entity> {
        self.try_index(pos).and_then(|index| self.cells[index])
    }

    #[inline]
    pub fn set(&mut self, pos: Position, entity: Option<Entity>) {
        if let Some(index) = self.try_index(pos) {
            self.cells[index] = entity;
        }
    }

    #[inline]
    fn try_index(&self, pos: Position) -> Option<usize> {
        if pos.x >= self.width || pos.y >= self.height {
            None
        } else {
            Some(pos.y * self.width + pos.x)
        }
    }

    /// Off-board cells count as occupied so movers never step outside.
    #[inline]
    pub fn is_occupied(&self, pos: Position) -> bool {
        self.try_index(pos)
            .map(|index| self.cells[index].is_some())
            .unwrap_or(true)
    }
}
