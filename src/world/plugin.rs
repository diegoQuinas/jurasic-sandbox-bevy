use bevy::{
    app::{App, Plugin, Startup},
    ecs::schedule::IntoScheduleConfigs,
};

use crate::app::StartupSet;

use super::{Board, Occupancy, spawn_board};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        // Insert before Startup: Commands.insert_resource would not be visible
        // to spawn_creatures in the same schedule without an extra ApplyDeferred.
        let (width, height) = {
            let board = app.world().resource::<Board>();
            (board.width, board.height)
        };
        app.insert_resource(Occupancy::new(width, height));
        app.add_systems(Startup, spawn_board.in_set(StartupSet::Board));
    }
}
