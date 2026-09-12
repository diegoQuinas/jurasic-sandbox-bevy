use bevy::app::{App, Plugin};

use super::{Board, Occupancy};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        // Insert before Startup so spawn_creatures can read Board/Occupancy
        // in the same schedule without an extra ApplyDeferred.
        let board = Board {
            width: 250,
            height: 250,
        };
        app.insert_resource(Occupancy::new(board.width, board.height));
        app.insert_resource(board);
    }
}
