use bevy::app::{App, Plugin};

use crate::config::Config;

use super::{Board, Occupancy};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        // Insert before Startup so spawn_creatures can read Board/Occupancy
        // in the same schedule without an extra ApplyDeferred.
        let config = app.world().resource::<Config>();
        let board = Board {
            width: config.simulation.world_width as usize,
            height: config.simulation.world_height as usize,
        };
        app.insert_resource(Occupancy::new(board.width, board.height));
        app.insert_resource(board);
    }
}
