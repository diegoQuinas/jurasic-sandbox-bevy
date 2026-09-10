use std::time::Instant;

use bevy::prelude::*;

use crate::{
    app::SystemPerformance,
    simulation::{
        components::{Decay, DinosaurStats, Egg, Health, Hunger, Mortal},
        spawn::{corpse_bundle, dinosaur_bundle},
    },
    world::{Occupancy, Position},
};

pub fn mature_eggs_system(
    mut commands: Commands,
    query: Query<(Entity, &mut Egg, &Position, &DinosaurStats)>,
    mut occupancy: ResMut<Occupancy>,
    mut performance: ResMut<SystemPerformance>,
) {
    let start = Instant::now();
    for (entity, mut egg, pos, genes) in query {
        if egg.age < 90 {
            egg.age = egg.age.saturating_add(1);
        } else {
            let dino = commands.spawn(dinosaur_bundle(pos.x, pos.y, genes)).id();
            occupancy.set(*pos, Some(dino));
            commands.entity(entity).despawn();
        }
    }
    performance.eggs_mature = start.elapsed().as_secs_f64() * 1000.00
}

pub fn hunger_system(query: Query<&mut Hunger>) {
    for mut hunger in query {
        hunger.increase(0.01);
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
    query: Query<(Entity, &Health, &Position), With<Mortal>>,
) {
    for (e, h, p) in query {
        if h.is_dead() {
            commands.spawn(corpse_bundle(p.x, p.y));
            commands.entity(e).despawn();
        }
    }
}

pub fn decay_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &mut Decay)>,
    mut occupancy: ResMut<Occupancy>,
) {
    for (e, p, mut d) in query {
        d.increase();

        if d.degradation >= d.degradation_threshold {
            occupancy.set(*p, None);
            commands.entity(e).despawn();
        }
    }
}
