use bevy::ecs::{
    bundle::Bundle,
    system::{Commands, Res, ResMut},
};
use rand::{RngExt, rng, rngs::ThreadRng};
use ratatui::style::Color;

use crate::world::{Board, Occupancy, Position, Renderable};

use super::components::*;

pub const STARTING_FAMILIES: u32 = 100;

fn create_families(rng: &mut ThreadRng) -> Vec<DinosaurStats> {
    let starting_generation_number = 0;

    (1..=STARTING_FAMILIES)
        .map(|_| {
            let color = (
                rng.random_range(0..=255),
                rng.random_range(0..=255),
                rng.random_range(0..=255),
            );
            let starvation_resistance = rng.random_range(-50..=50) as f64 / 100.0;
            DinosaurStats {
                generation: starting_generation_number,
                color,
                starvation_resistance,
            }
        })
        .collect()
}

pub fn spawn_creatures(
    board: Res<Board>,
    mut occupancy: ResMut<Occupancy>,
    mut commands: Commands,
) {
    let mut rng = rand::rng();
    let families = create_families(&mut rng);

    for family in families {
        let x = rng.random_range(0..board.width);
        let y = rng.random_range(0..board.height);
        let entity = commands.spawn(dinosaur_bundle(x, y, &family)).id();
        occupancy.set(Position { x, y, z: 3 }, Some(entity));
    }
}

pub fn dinosaur_bundle(x: usize, y: usize, genes: &DinosaurStats) -> impl Bundle {
    (
        Hunger(1.0),
        Health(1.0),
        Position { x, y, z: 3 },
        Renderable {
            glyph: "D",
            color: Color::Rgb(genes.color.0, genes.color.1, genes.color.2),
        },
        Mortal {},
        Herbivore {},
        Maturity(0.0),
        Desire(0.0),
        DinoState::default(),
        Gender(rng().random()),
        genes.clone(),
    )
}

pub fn egg_bundle(x: usize, y: usize, genes: DinosaurStats) -> impl Bundle {
    (
        Egg { age: 0 },
        Position { x, y, z: 1 },
        genes,
        Renderable {
            glyph: "0",
            color: Color::White,
        },
    )
}

pub fn corpse_bundle(x: usize, y: usize) -> impl Bundle {
    (
        Corpse,
        Renderable {
            glyph: "%",
            color: Color::Red,
        },
        Position { x, y, z: 0 },
        Decay {
            degradation_threshold: 10,
            degradation: 0,
        },
    )
}

pub fn plant_bundle(x: usize, y: usize) -> impl Bundle {
    let color = random_green();
    (
        Plant { health: 7 },
        Position { x, y, z: 2 },
        Renderable {
            glyph: "🌳",
            color: Color::Rgb(color.0, color.1, color.2),
        },
        Decay {
            degradation: 0,
            degradation_threshold: 1000,
        },
    )
}

pub fn grass_bundle(x: usize, y: usize) -> impl Bundle {
    let color = random_green();
    (
        Position { x, y, z: 1 },
        Renderable {
            glyph: r#"""#,
            color: Color::Rgb(color.0, color.1, color.2),
        },
        Decay {
            degradation: 0,
            degradation_threshold: 75,
        },
    )
}

fn random_green() -> (u8, u8, u8) {
    let mut rng = rand::rng();

    let r = rng.random_range(20..100);
    let g = rng.random_range(120..=255);
    let b = rng.random_range(20..100);

    (r, g, b)
}
