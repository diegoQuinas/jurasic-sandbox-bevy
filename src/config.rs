use bevy::ecs::resource::Resource;
use serde::Deserialize;

#[derive(Debug, Deserialize, Resource, Clone)]
pub struct Config {
    pub simulation: SimulationConfig,
    pub dinosaurs: DinosaurConfig,
}

impl Config {
    pub fn load(path: impl AsRef<std::path::Path>) -> color_eyre::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SimulationConfig {
    pub ticks_per_second: u32,
    pub world_width: u32,
    pub world_height: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DinosaurConfig {
    pub initial_population: usize,
}
