use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};

use crate::{
    app::{Performance, StartupSet, SystemPerformance, performance_system},
    simulation::CreaturesPlugin,
    ui::TuiPlugin,
    world::{Board, BoardPlugin},
};

mod app;
mod simulation;
mod ui;
mod world;

fn main() {
    App::new()
        .init_resource::<Performance>()
        .init_resource::<SystemPerformance>()
        .configure_sets(Startup, (StartupSet::Board, StartupSet::Creatures).chain())
        .add_systems(Update, performance_system)
        .add_plugins(BoardPlugin)
        .add_plugins(CreaturesPlugin)
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(1))))
        .add_plugins(TuiPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands, board: Res<Board>) {
    commands.insert_resource(ui::Camera::new(board.width / 2, board.height / 2));
}
