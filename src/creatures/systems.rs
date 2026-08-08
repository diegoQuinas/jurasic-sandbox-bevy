use std::time::Instant;

use bevy::prelude::*;
use rand::{RngExt, rng, seq::IndexedRandom};

use crate::{
    SystemPerformance,
    board::{Board, Occupancy, Position},
    creatures::{components::*, spawn::*},
};

pub fn wander_system(
    board: Res<Board>,
    mut performance: ResMut<SystemPerformance>,
    mut occupancy: ResMut<Occupancy>,
    mut dinos: Query<(Entity, &mut Position), With<Dinosaur>>,
) {
    let start = Instant::now();
    let mut rng = rand::rng();

    for (entity, mut position) in &mut dinos {
        if !rng.random_bool(0.5) {
            continue;
        }

        let directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];

        let direction = directions.choose(&mut rng).unwrap();
        let mut dx = 0;
        let mut dy = 0;
        match direction {
            Direction::East => dx = 1,
            Direction::West => dx = -1,
            Direction::North => dy = -1,
            Direction::South => dy = 1,
        }
        let target = Position {
            x: position.x.saturating_add_signed(dx),
            y: position.y.saturating_add_signed(dy),
        };

        if !board.is_inside(target) {
            continue;
        }
        if occupancy.is_occupied(target) {
            continue;
        }

        occupancy.move_entity(entity, *position, target);
        *position = target;
    }
    performance.wander = start.elapsed().as_secs_f64() * 1000.00
}

pub fn mature_eggs_system(
    mut commands: Commands,
    query: Query<(Entity, &mut Egg, &Position)>,
    mut performance: ResMut<SystemPerformance>,
) {
    let start = Instant::now();
    for (entity, mut egg, pos) in query {
        if egg.age < 10 {
            egg.age = egg.age.saturating_add(1);
        } else {
            commands.spawn(dinosaur_bundle(pos.x, pos.y));
            commands.entity(entity).despawn();
        }
    }
    performance.eggs_mature = start.elapsed().as_secs_f64() * 1000.00
}

pub fn hunger_system(
    mut commands: Commands,
    query: Query<(Entity, &mut Hungry)>,
    mut performance: ResMut<SystemPerformance>,
) {
    let start = Instant::now();
    for (entity, mut hungry) in query {
        hungry.increase(1);

        if hungry.hunger > hungry.starvation_threshold {
            commands.entity(entity).insert(Starving);
        } else {
            commands.entity(entity).remove::<Starving>();
        }
    }
    performance.hunger = start.elapsed().as_secs_f64() * 1000.00;
}

pub fn starving_system(
    query: Query<(&Starving, &mut Health)>,

    mut performance: ResMut<SystemPerformance>,
) {
    let start = Instant::now();
    for (_, mut h) in query {
        h.decrease(1);
    }
    performance.starving = start.elapsed().as_secs_f64() * 1000.00;
}

pub fn death_system(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Position), With<Mortal>>,
    mut performance: ResMut<SystemPerformance>,
) {
    let start = Instant::now();
    for (e, h, p) in query {
        if h.is_dead() {
            commands.spawn(corpse_bundle(p.x, p.y));
            commands.entity(e).despawn();
        }
    }
    performance.death = start.elapsed().as_secs_f64() * 1000.00
}

pub fn decay_system(
    mut commands: Commands,
    query: Query<(Entity, &mut Decay)>,
    mut performance: ResMut<SystemPerformance>,
) {
    let start = Instant::now();
    for (e, mut d) in query {
        d.increase();

        if d.degradation >= d.degradation_threshold {
            commands.entity(e).despawn();
        }
    }
    performance.decay = start.elapsed().as_secs_f64() * 1000.00
}

pub fn spawn_random_plants(
    mut commands: Commands,
    board: Res<Board>,
    mut performance: ResMut<SystemPerformance>,
) {
    let start = Instant::now();
    let x = rng().random_range(0..board.width);
    let y = rng().random_range(0..board.height);

    commands.spawn(plant_bundle(x, y));
    performance.spawn_plants = start.elapsed().as_secs_f64() * 1000.00
}

pub fn reproduction_system(
    mut commands: Commands,
    query: Query<&Position, With<Dinosaur>>,
    mut performance: ResMut<SystemPerformance>,
) {
    let start = Instant::now();
    for p in query {
        let reproduct = rng().random_bool(0.1);
        if !reproduct {
            return;
        }
        commands.spawn(egg_bundle(p.x, p.y));
    }
    performance.reproduction = start.elapsed().as_secs_f64() * 1000.00
}
