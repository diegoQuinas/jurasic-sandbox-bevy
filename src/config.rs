use bevy::ecs::resource::Resource;
use serde::Deserialize;

#[derive(Debug, Deserialize, Resource, Clone)]
pub struct Config {
    pub simulation: SimulationConfig,
    pub dinosaurs: DinosaurConfig,
    pub world: WorldConfig,
}

impl Config {
    pub fn load(path: impl AsRef<std::path::Path>) -> color_eyre::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SimulationConfig {
    pub sim_hz: f64,
    pub ticks_per_second: u32,
    pub world_width: usize,
    pub world_height: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DinosaurConfig {
    pub initial_population: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorldConfig {
    pub max_trees: usize,
    pub grass_density_rate: f64,
}
