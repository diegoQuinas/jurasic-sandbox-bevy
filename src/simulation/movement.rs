use bevy::prelude::*;

use rand::{RngExt, rng};

use crate::{
    simulation::Desire,
    world::{Position, WorldMap},
};

use super::components::{DinoState, DinosaurStats, Direction, Gender};

/// Metabolism 0..=1 maps to a 0.25..=1.0 chance to take a step this tick.
fn step_chance(metabolism: f64) -> f64 {
    0.25 + metabolism.clamp(0.0, 1.0) * 0.75
}

/// One Chebyshev step if the cell is free and metabolism grants this tick.
pub fn try_move(
    world: &mut WorldMap,
    entity: Entity,
    pos: &mut Position,
    dir: Direction,
    metabolism: f64,
) -> bool {
    let target = step(*pos, dir);
    if target == *pos || !world.is_free(target) {
        return false;
    }
    if !rng().random_bool(step_chance(metabolism)) {
        return false;
    }
    world.move_entity(entity, *pos, target);
    *pos = target;
    true
}

/// Chebyshev distance: a diagonal step counts as 1, matching 8-way movement.
pub fn chebyshev(a: &Position, b: &Position) -> usize {
    a.x.abs_diff(b.x).max(a.y.abs_diff(b.y))
}

pub fn step(pos: Position, dir: Direction) -> Position {
    let mut next = pos;
    match dir {
        Direction::North => next.y = next.y.saturating_sub(1),
        Direction::South => next.y = next.y.saturating_add(1),
        Direction::East => next.x = next.x.saturating_add(1),
        Direction::West => next.x = next.x.saturating_sub(1),
        Direction::NorthEast => {
            next.x = next.x.saturating_add(1);
            next.y = next.y.saturating_sub(1);
        }
        Direction::NorthWest => {
            next.x = next.x.saturating_sub(1);
            next.y = next.y.saturating_sub(1);
        }
        Direction::SouthEast => {
            next.x = next.x.saturating_add(1);
            next.y = next.y.saturating_add(1);
        }
        Direction::SouthWest => {
            next.x = next.x.saturating_sub(1);
            next.y = next.y.saturating_add(1);
        }
    }
    next
}

fn neighbor_free(pos: &Position, dir: Direction, world: &WorldMap) -> bool {
    world.is_free(step(*pos, dir))
}

/// `pos` itself plus its 8 neighbors — every cell within Chebyshev distance 1.
pub fn neighborhood(pos: Position) -> [Position; 9] {
    let mut cells = [pos; 9];
    for (i, dir) in Direction::ALL.into_iter().enumerate() {
        cells[i + 1] = step(pos, dir);
    }
    cells
}

fn dir_from_signs(sx: i8, sy: i8) -> Direction {
    match (sx, sy) {
        (1, 0) => Direction::East,
        (-1, 0) => Direction::West,
        (0, 1) => Direction::South,
        (0, -1) | (0, 0) => Direction::North,
        (1, -1) => Direction::NorthEast,
        (-1, -1) => Direction::NorthWest,
        (1, 1) => Direction::SouthEast,
        (-1, 1) => Direction::SouthWest,
        _ => Direction::North,
    }
}

/// Prefer a diagonal if both axes differ, then the cardinals. Always returns a
/// direction; the caller still has to check `is_free` before moving.
pub fn find_closest_tile(
    world: &WorldMap,
    dino_pos: &Position,
    target_pos: &Position,
) -> Direction {
    let dx = target_pos.x as isize - dino_pos.x as isize;
    let dy = target_pos.y as isize - dino_pos.y as isize;
    let sx = dx.signum() as i8;
    let sy = dy.signum() as i8;

    let mut candidates = Vec::with_capacity(3);
    if sx != 0 && sy != 0 {
        candidates.push(dir_from_signs(sx, sy));
    }
    if sx != 0 {
        candidates.push(dir_from_signs(sx, 0));
    }
    if sy != 0 {
        candidates.push(dir_from_signs(0, sy));
    }

    for dir in &candidates {
        if neighbor_free(dino_pos, *dir, world) {
            return *dir;
        }
    }

    candidates.into_iter().next().unwrap_or(Direction::North)
}

pub fn find_closest_target_reproduction(
    origin: &Position,
    origin_gender: &Gender,
    targets: &[(Entity, Position, DinosaurStats, DinoState, Gender, Desire)],
) -> Option<(Entity, Position, DinosaurStats, Desire)> {
    targets
        .iter()
        .filter(|(_, _, _, target_state, target_gender, _)| {
            target_gender.0 != origin_gender.0 && *target_state == DinoState::SeekingPartner
        })
        .min_by_key(|(_, pos, _, _, _, _)| chebyshev(origin, pos))
        .map(|(e, p, s, _, _, d)| (*e, *p, *s, *d))
}
