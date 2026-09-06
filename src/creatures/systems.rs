use std::time::Instant;

use bevy::prelude::*;
use rand::{RngExt, rng, seq::SliceRandom};

use crate::{
    SystemPerformance,
    board::{Board, Occupancy, Position},
    creatures::{components::*, spawn::*},
};

pub fn wander_system(
    board: Res<Board>,
    mut performance: ResMut<SystemPerformance>,
    mut occupancy: ResMut<Occupancy>,
    mut dinos: Query<(Entity, &mut Position), (With<Wanderer>,Without<Plant>)>,
    mut plants_query: Query<(Entity, &Position), (With<Plant>,Without<Dinosaur>)>,
) {
    let start = Instant::now();
    let mut rng = rand::rng();

    let mut directions = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ];


    for (entity, mut position) in &mut dinos {
        let mut target = Position {
            x: position.x,
            y: position.y,
            z: position.z,
        };
       
        if let Some((_plant_entity, plant_pos)) = find_closest_food(&position, &plants_query) {
            let move_direction = find_closest_tide(&position, &plant_pos);
            match move_direction {
            Direction::North => target.y = target.y.saturating_sub(1),
            Direction::South => target.y = target.y.saturating_add(1),
            Direction::East => target.x = target.x.saturating_add(1),
            Direction::West => target.x = target.x.saturating_sub(1),
            }
        }
        
        if target.x != position.x || target.y != position.y {
            occupancy.move_entity(entity, *position, target);
            *position = target;
        }

    }
    performance.wander = start.elapsed().as_secs_f64() * 1000.00
}
//Recive un dinasurio y devuelve la planta mas sercana. 
pub fn find_closest_food(dino_pos: &Position,plants_query: &Query<(Entity, &Position), With<Plant>>,)-> Option<(Entity, Position)> {
    plants_query
        .iter()
        .min_by_key(|(_, plant_pos)| {
            // Calculate Manhattan distance on a grid
            let distance_x = dino_pos.x.abs_diff(plant_pos.x);
            let distance_y = dino_pos.y.abs_diff(plant_pos.y);
            
            distance_x + distance_y
        }).map(|(entity, pos)| (entity, *pos))

}
pub fn find_closest_tide(wander_pos: &Position, target_pos: &Position) -> Direction {
    let dx = target_pos.x as isize - wander_pos.x as isize;
    let dy = target_pos.y as isize - wander_pos.y as isize;

    if dx.abs() > dy.abs() {
        if dx > 0 {
            Direction::East
        } else {
            Direction::West
        }
    } else {
        if dy > 0 {
            Direction::South
        } else {
            Direction::North
        }
    }
}

pub fn mature_eggs_system(
    mut commands: Commands,
    query: Query<(Entity, &mut Egg, &Position, &Genes)>,
    mut performance: ResMut<SystemPerformance>,
) {
    let start = Instant::now();
    for (entity, mut egg, pos, genes) in query {
        if egg.age < 10 {
            egg.age = egg.age.saturating_add(1);
        } else {
            commands.spawn(dinosaur_bundle(pos.x, pos.y, genes));
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
    mut occupancy: ResMut<Occupancy>,
) {
    let start = Instant::now();
    let x = rng().random_range(0..board.width);
    let y = rng().random_range(0..board.height);

    let entity = commands.spawn(plant_bundle(x, y)).id();
    occupancy.set(Position { x, y, z: 0 }, Some(entity));
    performance.spawn_plants = start.elapsed().as_secs_f64() * 1000.00
}

pub fn reproduction_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &Genes), With<Dinosaur>>,
    mut performance: ResMut<SystemPerformance>,
    mut occupancy: ResMut<Occupancy>,
) {
    let start = Instant::now();
    for (entity, p, genes) in query {
        let reproduct = rng().random_bool(0.1);
        if !reproduct {
            return;
        }

        commands.spawn(egg_bundle(p.x, p.y, genes.clone()));
        occupancy.set(*p, Some(entity));
    }
    performance.reproduction = start.elapsed().as_secs_f64() * 1000.00
}
