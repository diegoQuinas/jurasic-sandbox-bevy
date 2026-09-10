use std::time::Instant;

use bevy::prelude::*;

use crate::{
    app::SystemPerformance,
    simulation::{
        DinoState, Maturity,
        components::{Decay, DinosaurStats, Egg, Health, Hunger, Mortal},
        spawn::{corpse_bundle, dinosaur_bundle},
    },
    world::{Occupancy, Position, WorldMap},
};

pub fn maturing_system(mut query: Query<&mut Maturity>) {
    for mut maturity in &mut query {
        maturity.increase(0.001);
    }
}

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
    query: Query<(Entity, &Position, &DinoState), With<Mortal>>,
    mut world: WorldMap,
) {
    for (e, p, s) in query {
        if *s != DinoState::Dieing {
            continue;
        } else {
            let new_corpse = commands.spawn(corpse_bundle(p.x, p.y)).id();
            world.set_free(*p);
            commands.entity(e).despawn();
            world.set_occupied(new_corpse, *p);
        }
    }
}

pub fn healing_system(query: Query<(&mut Health, &DinoState)>) {
    for (mut health, state) in query {
        if *state != DinoState::Healing {
            continue;
        } else {
            health.increase(0.01);
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
