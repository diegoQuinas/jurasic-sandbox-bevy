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
    // Leemos la variable de entorno SIM_HZ para saber a qué frecuencia (ticks
    // por segundo) tiene que correr la simulación.
    //
    // std::env::var devuelve un Result: Ok(texto) si la variable existe,
    // o Err(...) si no está definida. Con `match` manejamos los dos casos.
    #[allow(clippy::option_if_let_else)]
    let sim_hz = match std::env::var("SIM_HZ") {
        // La variable existe: intentamos convertir el texto a número (f64).
        // Si el texto no es un número válido (ej: "abc"), parse() falla y
        // usamos 120.0 como respaldo gracias a unwrap_or.
        Ok(texto) => texto.parse().unwrap_or(120.0),
        // La variable no está definida: usamos 120.0 por defecto.
        Err(_) => 120.0,
    };

    let config = Config::load("config.toml")?;

    App::new()
        .init_resource::<Performance>()
        .init_resource::<SystemPerformance>()
        .insert_resource(config.clone())
        .insert_resource(Time::<Fixed>::from_hz(sim_hz))
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
