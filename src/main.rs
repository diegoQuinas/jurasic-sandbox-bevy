use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};

use crate::{board::BoardPlugin, creatures::CreaturesPlugin, terminal::TuiPlugin};
mod board;
mod creatures;
mod terminal;

fn main() {
    App::new()
        .init_resource::<Performance>()
        .init_resource::<SystemPerformance>()
        .configure_sets(Startup, (StartupSet::Board, StartupSet::Creatures).chain())
        .add_systems(Update, performance_system)
        .add_plugins(BoardPlugin)
        .add_plugins(CreaturesPlugin)
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(1000))),
        )
        .add_plugins(TuiPlugin)
        .run();
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum StartupSet {
    Board,
    Creatures,
}

#[derive(Resource, Default)]
pub struct Performance {
    pub ticks: u64,
    pub elapsed: f32,
    pub ticks_per_second: f32,
}
#[derive(Resource, Default)]
pub struct SystemPerformance {
    pub eggs_mature: f64,
    pub wander: f64,
    pub reproduction: f64,
    pub hunger: f64,
    pub starving: f64,
    pub death: f64,
    pub spawn_plants: f64,
    pub decay: f64,
}

pub fn performance_system(time: Res<Time>, mut performance: ResMut<Performance>) {
    performance.ticks += 1;
    performance.elapsed += time.delta_secs();

    if performance.elapsed >= 1.0 {
        performance.ticks_per_second = performance.ticks as f32 / performance.elapsed;

        performance.ticks = 0;
        performance.elapsed = 0.0;
    }
}
