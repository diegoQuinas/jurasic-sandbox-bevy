use bevy::ecs::{
    bundle::Bundle,
    system::{Commands, Res},
};
use rand::{RngExt, rngs::ThreadRng};
use ratatui::style::Color;

use crate::{
    board::{Board, Position, Renderable},
    creatures::components::*,
};

pub const STARTING_FAMILIES: u32 = 1;

fn create_families(rng: &mut ThreadRng) -> Vec<Genes> {
    (1..=STARTING_FAMILIES)
        .map(|id| {
            let color = Color::Rgb(
                rng.random_range(0..=255),
                rng.random_range(0..=255),
                rng.random_range(0..=255),
            );
            Genes {
                starving_resistance: rng.random_range(-50..=50),
                color,
                family: FamilyId(id),
            }
        })
        .collect()
}

pub fn spawn_creatures(board: Res<Board>, mut commands: Commands) {
    let mut rng = rand::rng();
    let families = create_families(&mut rng);

    for family in families {
        let x = rng.random_range(0..board.width);
        let y = rng.random_range(0..board.height);
        commands.spawn(dinosaur_bundle(x, y, &family));
    }
}

pub fn dinosaur_bundle(x: usize, y: usize, genes: &Genes) -> impl Bundle {
    let n = 75;
    let n = (n + genes.starving_resistance).clamp(25, 125) as u32;
    (
        Dinosaur {},
        Hungry {
            hunger: 0,
            starvation_threshold: n,
        },
        Health(10),
        Position { x, y, z: 3 },
        Renderable {
            glyph: "D",
            color: genes.color,
        },
        Mortal {},
        Wanderer {},
        genes.clone(),
    )
}

pub fn egg_bundle(x: usize, y: usize, genes: Genes) -> impl Bundle {
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
    (
        Plant { health: 7 },
        Position { x, y, z: 2 },
        Renderable {
            glyph: "🌳",
            color: Color::Green,
        },
        Decay {
            degradation: 0,
            degradation_threshold: 100,
        },
    )
}
