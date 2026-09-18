use bevy::prelude::*;
use rand::{RngExt, rng, seq::SliceRandom};

use crate::{
    simulation::{
        Corpse, DinoState, Direction, Maturity,
        components::{Decay, DinosaurStats, Egg, Health, Hunger, Plant},
        movement::{step, try_move},
        spawn::{corpse_bundle, dinosaur_bundle},
    },
    world::{Occupancy, Position, Renderable, WorldMap},
};

pub fn maturing_system(query: Query<(&mut Maturity, &DinosaurStats)>) {
    let mut rng = rng();
    for (mut maturity, stats) in query {
        let random_multiplier = rng.random_range(0.0..=10.0);
        let increasement = stats.metabolism * 0.0001 * random_multiplier;
        maturity.increase(increasement);
    }
}

pub fn mature_eggs_system(
    mut commands: Commands,
    query: Query<(Entity, &mut Egg, &Position, &DinosaurStats)>,
    mut world: WorldMap,
) {
    for (entity, mut egg, pos, genes) in query {
        if egg.age < 90 {
            egg.age = egg.age.saturating_add(1);
        } else if world.is_free(*pos) {
            let dino = commands.spawn(dinosaur_bundle(pos.x, pos.y, genes)).id();
            world.set_occupied(dino, *pos);
            commands.entity(entity).despawn();
        }
    }
}

pub fn hunger_system(query: Query<(&mut Hunger, &DinosaurStats)>) {
    for (mut hunger, stats) in query {
        let increasement = stats.metabolism * 0.005;
        hunger.increase(increasement);
    }
}

pub fn starving_system(query: Query<(&Hunger, &DinosaurStats, &mut Health)>) {
    for (hunger, dino_stats, mut health) in query {
        if hunger.hunger() > dino_stats.starvation_resistance {
            health.decrease(0.01);
        }
    }
}

pub fn death_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &DinoState, &DinosaurStats)>,
    mut world: WorldMap,
) {
    for (e, p, s, d_s) in query {
        if *s != DinoState::Dieing {
            continue;
        }
        commands.spawn(corpse_bundle(p.x, p.y, d_s.color));
        world.set_free(*p);
        commands.entity(e).despawn();
    }
}

pub fn healing_system(query: Query<(&mut Health, &DinoState)>) {
    for (mut health, state) in query {
        if *state != DinoState::Healing {
            continue;
        }
        health.increase(0.01);
    }
}

fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * 2.0f64.mul_add(-t, 3.0)
}

fn lerp_rgb(from: (f64, f64, f64), to: (f64, f64, f64), t: f64) -> (u8, u8, u8) {
    let t = smoothstep(t);
    (
        (to.0 - from.0).mul_add(t, from.0) as u8,
        (to.1 - from.1).mul_add(t, from.1) as u8,
        (to.2 - from.2).mul_add(t, from.2) as u8,
    )
}

/// Olive-brown rot that still reads as a sick version of the living color.
fn rotten_from(original: (f64, f64, f64)) -> (f64, f64, f64) {
    (
        72.0f64.mul_add(0.45, original.0 * 0.35),
        88.0f64.mul_add(0.40, original.1 * 0.40),
        38.0f64.mul_add(0.35, original.2 * 0.20),
    )
}

pub fn degrade_corpose_color_system(query: Query<(&Decay, &Corpse, &mut Renderable)>) {
    const BONE: (f64, f64, f64) = (176.0, 176.0, 176.0);
    const ROTTEN_END: f64 = 0.06;
    const BONE_END: f64 = 0.16;
    for (decay, corpse, mut renderable) in query {
        let degradation = decay.degradation.max(0.0);
        let original = (
            corpse.original_color.0 as f64,
            corpse.original_color.1 as f64,
            corpse.original_color.2 as f64,
        );
        let rotten = rotten_from(original);

        let color = if degradation < ROTTEN_END {
            lerp_rgb(original, rotten, degradation / ROTTEN_END)
        } else if degradation < BONE_END {
            lerp_rgb(
                rotten,
                BONE,
                (degradation - ROTTEN_END) / (BONE_END - ROTTEN_END),
            )
        } else {
            (BONE.0 as u8, BONE.1 as u8, BONE.2 as u8)
        };

        renderable.color = color;
    }
}

pub fn degrade_plant_color_system(query: Query<(&Decay, &Plant, &mut Renderable)>) {
    for (decay, plant, mut renderable) in query {
        let t = decay.degradation.clamp(0.0, 1.0);
        let original = (
            plant.original_color.0 as f64,
            plant.original_color.1 as f64,
            plant.original_color.2 as f64,
        );
        let target = (140.0, 110.0, 40.0);
        renderable.color = lerp_rgb(original, target, t);
    }
}

pub fn decay_system(
    mut commands: Commands,
    mut corpses: Query<(Entity, &Position, &mut Decay), With<Corpse>>,
    mut other: Query<(Entity, &Position, &mut Decay), Without<Corpse>>,
    mut occupancy: ResMut<Occupancy>,
) {
    let mut rng = rng();
    for (e, p, mut d) in &mut corpses {
        d.increase(rng.random_range(0.004..=0.02));
        if d.degradation >= d.degradation_threshold {
            occupancy.set(*p, None);
            commands.entity(e).despawn();
        }
    }
    for (e, p, mut d) in &mut other {
        d.increase(rng.random_range(0.00001..=0.0001));
        if d.degradation >= d.degradation_threshold {
            occupancy.set(*p, None);
            commands.entity(e).despawn();
        }
    }
}

pub fn wander_system(
    query: Query<(Entity, &mut Position, &DinoState, &DinosaurStats)>,
    mut world: WorldMap,
) {
    let mut rng = rng();
    for (entity, mut pos, dino_state, stats) in query {
        if *dino_state != DinoState::Wandering {
            continue;
        }

        let mut shuffled_directions = Direction::ALL;
        shuffled_directions.shuffle(&mut rng);

        let Some(dir) = shuffled_directions
            .into_iter()
            .find(|dir| world.is_free(step(*pos, *dir)))
        else {
            continue;
        };
        try_move(&mut world, entity, &mut pos, dir, stats.metabolism);
    }
}
