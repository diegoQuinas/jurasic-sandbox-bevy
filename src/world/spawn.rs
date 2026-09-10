use bevy::ecs::system::Commands;

use super::Board;

pub fn spawn_board(mut commands: Commands) {
    commands.spawn(Board {
        width: 50,
        height: 50,
    });
}
