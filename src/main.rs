use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};

use crate::{
    app::{Performance, StartupSet, SystemPerformance, performance_system},
    config::Config,
    simulation::CreaturesPlugin,
    ui::TuiPlugin,
    world::{Board, BoardPlugin},
};

mod app;
mod config;
mod simulation;
mod ui;
mod world;

fn main() -> color_eyre::Result<()> {
    let config = Config::load("config.toml")?;

    App::new()
        .init_resource::<Performance>()
        .init_resource::<SystemPerformance>()
        .insert_resource(config.clone())
        .insert_resource(Time::<Fixed>::from_hz(config.simulation.sim_hz))
        .configure_sets(Startup, (StartupSet::Board, StartupSet::Creatures).chain())
        .add_systems(Update, performance_system)
        .add_plugins(BoardPlugin)
        .add_plugins(CreaturesPlugin)
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / config.simulation.ticks_per_second as f64,
            ))),
        )
        .add_plugins(TuiPlugin)
        .add_systems(Startup, setup_camera)
        .run();

    Ok(())
}

fn setup_camera(mut commands: Commands, board: Res<Board>) {
    commands.insert_resource(ui::Camera::new(board.width / 2, board.height / 2));
}
