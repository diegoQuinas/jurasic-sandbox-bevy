use bevy::prelude::*;

#[derive(Resource)]
pub struct Board {
    pub width: usize,
    pub height: usize,
}

impl Board {
    pub fn is_inside(&self, position: Position) -> bool {
        position.x < self.width && position.y < self.height
    }
}

/// Grid cell. `z` is only a draw layer (who is on top), not occupancy.
#[derive(Component, Hash, PartialEq, Eq, Copy, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: &'static str,
    pub color: (u8, u8, u8),
}
