use bevy::ecs::{
    entity::Entity,
    system::{Res, ResMut, SystemParam},
};

use super::{Board, Occupancy, Position};

/// Bundles board bounds + occupancy so systems don't fight over those two resources.
#[derive(SystemParam)]
pub struct WorldMap<'w> {
    board: Res<'w, Board>,
    occupancy: ResMut<'w, Occupancy>,
}

impl<'w> WorldMap<'w> {
    pub fn is_free(&self, pos: Position) -> bool {
        self.board.is_inside(pos) && !self.occupancy.is_occupied(pos)
    }

    /// `None` for both an empty tile and an out-of-bounds position.
    pub fn entity_at(&self, pos: Position) -> Option<Entity> {
        self.occupancy.get(pos)
    }

    pub fn move_entity(&mut self, entity: Entity, from: Position, to: Position) {
        self.occupancy.set(from, None);
        self.occupancy.set(to, Some(entity));
    }

    pub fn set_occupied(&mut self, entity: Entity, pos: Position) {
        self.occupancy.set(pos, Some(entity));
    }

    pub fn set_free(&mut self, pos: Position) {
        self.occupancy.set(pos, None);
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.board.width, self.board.height)
    }
}
