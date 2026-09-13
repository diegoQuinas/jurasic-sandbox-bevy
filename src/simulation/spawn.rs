use bevy::ecs::{
    bundle::Bundle,
    system::{Commands, Res, ResMut},
};
use bevy::prelude::Resource;
use noise::{Fbm, NoiseFn, Perlin};
use rand::{Rng, RngExt, rng, rngs::ThreadRng, seq::IndexedRandom};

use crate::world::{Board, Occupancy, Position, Renderable};

use super::components::*;

pub const STARTING_FAMILIES: u32 = 100;

/// Low-frequency Perlin (FBM) that marks forest groves. Same field is used at
/// startup and when trees grow back, so new plants keep clustering.
#[derive(Resource, Clone)]
pub struct ForestNoise {
    noise: Fbm<Perlin>,
    threshold: f64,
}

impl ForestNoise {
    pub fn new(seed: u32) -> Self {
        let mut noise = Fbm::<Perlin>::new(seed);
        noise.octaves = 4;
        noise.frequency = 0.032;
        noise.lacunarity = 2.1;
        noise.persistence = 0.52;
        Self {
            noise,
            threshold: 0.18,
        }
    }

    /// 0 outside groves, up to 1 in the densest core.
    pub fn grove_density(&self, x: usize, y: usize) -> f64 {
        let n = self.noise.get([x as f64, y as f64]);
        if n <= self.threshold {
            0.0
        } else {
            ((n - self.threshold) / (1.0 - self.threshold)).clamp(0.0, 1.0)
        }
    }

    pub fn should_spawn_tree(&self, x: usize, y: usize, rng: &mut impl Rng) -> bool {
        let density = self.grove_density(x, y);
        density > 0.0 && rng.random_bool(0.15 + 0.75 * density)
    }
}

fn create_families(rng: &mut ThreadRng) -> Vec<DinosaurStats> {
    let starting_generation_number = 0;

    (1..=STARTING_FAMILIES)
        .map(|_| {
            let redness = rng.random_range(25..=255);
            let metabolism = (redness as f64) / 255.0; // More hungry ones are red
            let color = (
                redness,
                rng.random_range(0..=255),
                rng.random_range(0..=255),
            );
            let starvation_resistance: f64 = rng.random_range(0.5..1.0);
            DinosaurStats {
                generation: starting_generation_number,
                color,
                starvation_resistance,
                metabolism,
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
    let forest = ForestNoise::new(rng.random());

    for family in families {
        let x = rng.random_range(0..board.width);
        let y = rng.random_range(0..board.height);
        let entity = commands.spawn(dinosaur_bundle(x, y, &family)).id();
        occupancy.set(Position { x, y, z: 3 }, Some(entity));
    }

    for x in 0..board.width {
        for y in 0..board.height {
            if rng.random_bool(0.3) {
                commands.spawn(grass_bundle(x, y));
            }
            if forest.should_spawn_tree(x, y, &mut rng) {
                let pos = Position { x, y, z: 2 };
                if occupancy.is_occupied(pos) {
                    continue;
                }
                let entity = commands.spawn(plant_bundle(x, y)).id();
                occupancy.set(pos, Some(entity));
            }
        }
    }

    commands.insert_resource(forest);
}

pub fn dinosaur_bundle(x: usize, y: usize, genes: &DinosaurStats) -> impl Bundle {
    (
        Hunger(0.0),
        Health(1.0),
        Position { x, y, z: 3 },
        Renderable {
            glyph: "D",
            color: genes.color,
        },
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
            color: (255, 255, 255),
        },
    )
}

pub fn corpse_bundle(x: usize, y: usize, original_color: (u8, u8, u8)) -> impl Bundle {
    (
        Corpse { original_color },
        Renderable {
            glyph: "%",
            color: original_color,
        },
        Position { x, y, z: 0 },
        Decay {
            degradation_threshold: 4.0,
            degradation: 0.0,
        },
    )
}

pub fn plant_bundle(x: usize, y: usize) -> impl Bundle {
    let color = random_green();
    let glyph = random_plant_glyph();
    (
        Plant {
            health: 7,
            original_color: color,
        },
        Position { x, y, z: 2 },
        Renderable { glyph, color },
        Decay {
            degradation: 0.0,
            degradation_threshold: 1.0,
        },
    )
}

pub fn grass_bundle(x: usize, y: usize) -> impl Bundle {
    let color = random_green();
    let glyph = random_grass_glyph();
    (Grass, Position { x, y, z: 1 }, Renderable { glyph, color })
}

pub fn random_grass_glyph() -> &'static str {
    let glyphs = [".", ",", ",", "·", "'", "˙"];
    let mut rng = rng();
    glyphs.choose(&mut rng).unwrap()
}

pub fn random_plant_glyph() -> &'static str {
    let mut rng = rng();
    let glyphs = ["♣", "♠", "♧", "♤", "♣"];
    glyphs.choose(&mut rng).unwrap()
}

fn random_green() -> (u8, u8, u8) {
    let mut rng = rand::rng();

    let r = rng.random_range(20..100);
    let g = rng.random_range(120..=255);
    let b = rng.random_range(20..100);

    (r, g, b)
}
