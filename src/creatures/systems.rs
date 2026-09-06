use std::time::Instant;

use bevy::prelude::*;
use rand::{RngExt, rng};

use crate::{
    SystemPerformance,
    board::{Board, Occupancy, Position},
    creatures::{components::*, spawn::*},
};

pub fn wander_system(
    board: Res<Board>,
    mut performance: ResMut<SystemPerformance>,
    mut occupancy: ResMut<Occupancy>,
    mut dinos: Query<(Entity, &mut Position), (With<Wanderer>, With<Dinosaur>, Without<Plant>)>,
    plants_query: Query<(Entity, &Position), (With<Plant>, Without<Dinosaur>)>,
) {
    let start = Instant::now();

    for (entity, mut position) in &mut dinos {
        let Some((_plant_entity, plant_pos)) = find_closest_food(&position, &plants_query) else {
            continue;
        };

        let move_direction = find_closest_tile(&position, &plant_pos, &board, &occupancy);
        let target = step(*position, move_direction);

        if target != *position && board.is_inside(target) && !occupancy.is_occupied(target) {
            occupancy.set(*position, None);
            occupancy.set(target, Some(entity));
            *position = target;
        }
    }
    performance.wander = start.elapsed().as_secs_f64() * 1000.00
}
//Recive un dinasurio y devuelve la planta mas sercana.
pub fn find_closest_food(
    dino_pos: &Position,
    plants_query: &Query<(Entity, &Position), (With<Plant>, Without<Dinosaur>)>,
) -> Option<(Entity, Position)> {
    plants_query
        .iter()
        .min_by_key(|(_, plant_pos)| {
            // Calculate Manhattan distance on a grid
            let distance_x = dino_pos.x.abs_diff(plant_pos.x);
            let distance_y = dino_pos.y.abs_diff(plant_pos.y);

            distance_x + distance_y
        })
        .map(|(entity, pos)| (entity, *pos))
}

pub fn find_closest_tile(
    wander_pos: &Position,
    target_pos: &Position,
    board: &Board,
    occupancy: &Occupancy,
) -> Direction {
    let dx = target_pos.x as isize - wander_pos.x as isize;
    let dy = target_pos.y as isize - wander_pos.y as isize;

    // Primera opción
    let primary = if dx.abs() > dy.abs() {
        if dx > 0 {
            Direction::East
        } else {
            Direction::West
        }
    } else if dy > 0 {
        Direction::South
    } else {
        Direction::North
    };

    if neighbor_free(wander_pos, primary, board, occupancy) {
        return primary;
    }

    // Fallback
    let secondary = if dx.abs() > dy.abs() {
        if dy > 0 {
            Direction::South
        } else if dy < 0 {
            Direction::North
        } else {
            primary
        }
    } else if dx > 0 {
        Direction::East
    } else if dx < 0 {
        Direction::West
    } else {
        primary
    };

    // Como la función debe retornar obligatoriamente una direccion, devuelve primary si secondary esta ocupado aunque lo detiene arriba el wanderer_system
    if secondary != primary && neighbor_free(wander_pos, secondary, board, occupancy) {
        secondary
    } else {
        primary
    }
}

fn step(pos: Position, dir: Direction) -> Position {
    let mut next = pos;
    match dir {
        Direction::North => next.y = next.y.saturating_sub(1),
        Direction::South => next.y = next.y.saturating_add(1),
        Direction::East => next.x = next.x.saturating_add(1),
        Direction::West => next.x = next.x.saturating_sub(1),
    }
    next
}

fn neighbor_free(pos: &Position, dir: Direction, board: &Board, occupancy: &Occupancy) -> bool {
    let next = step(*pos, dir);
    board.is_inside(next) && !occupancy.is_occupied(next)
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
    query: Query<(Entity, &Position, &mut Decay)>,
    mut performance: ResMut<SystemPerformance>,
    mut occupancy: ResMut<Occupancy>,
) {
    let start = Instant::now();
    for (e, p, mut d) in query {
        d.increase();

        if d.degradation >= d.degradation_threshold {
            occupancy.set(*p, None);
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
    query: Query<(Entity, &Position, &Genes), (With<Dinosaur>, Without<Starving>)>,
    mut performance: ResMut<SystemPerformance>,
    mut occupancy: ResMut<Occupancy>,
) {
    let start = Instant::now();
    for (entity, p, genes) in query {
        let reproduct = rng().random_bool(0.1);
        if !reproduct {
            continue;
        }

        commands.spawn(egg_bundle(p.x, p.y, genes.clone()));
        occupancy.set(*p, Some(entity));
    }
    performance.reproduction = start.elapsed().as_secs_f64() * 1000.00
}
