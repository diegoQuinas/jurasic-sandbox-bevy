use bevy::prelude::*;

use crate::world::{Position, WorldMap};

use super::components::{DinoState, DinosaurStats, Direction, Gender};

pub fn manhattan(a: &Position, b: &Position) -> usize {
    a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
}

pub fn step(pos: Position, dir: Direction) -> Position {
    let mut next = pos;
    match dir {
        Direction::North => next.y = next.y.saturating_sub(1),
        Direction::South => next.y = next.y.saturating_add(1),
        Direction::East => next.x = next.x.saturating_add(1),
        Direction::West => next.x = next.x.saturating_sub(1),
    }
    next
}

fn neighbor_free(pos: &Position, dir: Direction, world: &WorldMap) -> bool {
    world.is_free(step(*pos, dir))
}

/// Greedy step toward `target_pos`: longer axis first, then the other if blocked.
/// Always returns a direction even if both neighbors are occupied (caller must
/// still check `is_free` before moving).
pub fn find_closest_tile(world: &WorldMap, dino_pos: &Position, target_pos: &Position) -> Direction {
    let dx = target_pos.x as isize - dino_pos.x as isize;
    let dy = target_pos.y as isize - dino_pos.y as isize;

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

    if neighbor_free(dino_pos, primary, world) {
        return primary;
    }

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

    if secondary != primary && neighbor_free(dino_pos, secondary, world) {
        secondary
    } else {
        primary
    }
}

pub fn find_closest_target_reproduction(
    origin: &Position,
    origin_gender: &Gender,
    targets: &[(Entity, Position, DinosaurStats, DinoState, Gender)],
) -> Option<(Entity, Position, DinosaurStats)> {
    targets
        .iter()
        .filter(|(_, _, _, target_state, target_gender)| {
            target_gender.0 != origin_gender.0 && *target_state == DinoState::SeekingPartner
        })
        .min_by_key(|(_, pos, _, _, _)| manhattan(origin, pos))
        .map(|(e, p, s, _, _)| (*e, *p, *s))
}
