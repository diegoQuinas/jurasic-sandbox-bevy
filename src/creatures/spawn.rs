use bevy::ecs::{
    bundle::Bundle,
    system::{Commands, Res},
};
use rand::RngExt;
use ratatui::style::Color;

use crate::{
    board::{Board, Position, Renderable},
    creatures::components::*,
};

const FAMILY_COUNT: u32 = 10;

const PALETTE: [Color; 10] = [
    Color::LightRed,
    Color::LightGreen,
    Color::LightYellow,
    Color::LightBlue,
    Color::LightMagenta,
    Color::LightCyan,
    Color::White,
    Color::Red,
    Color::Green,
    Color::Blue,
];

fn create_families() -> Vec<Genes> {
    let mut rng = rand::rng();

    (1..=FAMILY_COUNT)
        .map(|id| {
            let color = PALETTE[(id as usize - 1) % PALETTE.len()];
            Genes {
                starving_resistance: rng.random_range(-5..=5),
                glyph: String::from("D "),
                color,
                family: FamilyId(id),
            }
        })
        .collect()
}

pub fn spawn_creatures(board: Res<Board>, mut commands: Commands) {
    let mut rng = rand::rng();
    let mut families = create_families();

    for y in 0..board.height {
        for x in 0..board.width {
            if rng.random_range(0..10) != 0 {
                continue;
            }

            let Some(family) = families.pop() else {
                return;
            };

            commands.spawn(dinosaur_bundle(x, y, &family));
        }
    }
}

pub fn dinosaur_bundle(x: usize, y: usize, genes: &Genes) -> impl Bundle {
    let n = 15;
    let n = (n + genes.starving_resistance).clamp(10, 20) as u32;
    (
        Dinosaur {},
        Hungry {
            hunger: 0,
            starvation_threshold: n,
        },
        Health(10),
        Position { x, y, z: 3 },
        Renderable {
            glyph: genes.glyph.clone(),
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
            glyph: String::from("0"),
            color: Color::White,
        },
    )
}

pub fn corpse_bundle(x: usize, y: usize) -> impl Bundle {
    (
        Corpse,
        Renderable {
            glyph: String::from("%"),
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
        Plant,
        Position { x, y, z: 2 },
        Renderable {
            glyph: String::from("♣"),
            color: Color::Green,
        },
        Decay {
            degradation: 0,
            degradation_threshold: 100,
        },
    )
}
