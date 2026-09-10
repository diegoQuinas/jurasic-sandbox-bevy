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
        .insert_resource(Board {
            width: 40,
            height: 30,
        })
        .init_resource::<Performance>()
        .init_resource::<SystemPerformance>()
        .configure_sets(Startup, (StartupSet::Board, StartupSet::Creatures).chain())
        .add_systems(Update, performance_system)
        .add_plugins(BoardPlugin)
        .add_plugins(CreaturesPlugin)
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(100))))
        .add_plugins(TuiPlugin)
        .run();
}
