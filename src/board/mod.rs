use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::{
        component::Component,
        entity::Entity,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Res},
    },
};

use crate::{StartupSet, board::spawn::spawn_board};

pub mod render;
pub mod spawn;

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

#[derive(Component, Hash, PartialEq, Eq, Copy, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: String,
}
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

    #[inline]
    pub fn get(&self, pos: Position) -> Option<Entity> {
        self.cells[self.index(pos)]
    }

    #[inline]
    pub fn set(&mut self, pos: Position, entity: Option<Entity>) {
        let index = self.index(pos);
        self.cells[index] = entity;
    }

    #[inline]
    pub fn index(&self, pos: Position) -> usize {
        pos.y * self.width + pos.x
    }

    #[inline]
    pub fn is_occupied(&self, pos: Position) -> bool {
        self.cells[self.index(pos)].is_some()
    }
}

pub fn setup_occupancy(mut commands: Commands, board: Res<Board>) {
    commands.insert_resource(Occupancy::new(board.width, board.height));
}

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_board.in_set(StartupSet::Board),
                setup_occupancy.after(spawn_board),
            ),
        )
        .add_systems(Update, render::render);
    }
}
