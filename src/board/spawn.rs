use bevy::ecs::system::Commands;

use crate::board::Board;

pub fn spawn_board(mut commands: Commands) {
    commands.spawn(Board {
        width: 1_000,
        height: 1_000,
    });
}
