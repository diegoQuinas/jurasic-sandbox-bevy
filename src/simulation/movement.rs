use bevy::prelude::*;

use rand::{RngExt, rng};

use crate::world::{Position, WorldMap};

use super::components::Direction;

/// Metabolism 0..=1 maps to a 0.25..=1.0 chance to take a step this tick.
const fn step_chance(metabolism: f64) -> f64 {
    metabolism.clamp(0.0, 1.0).mul_add(0.75, 0.25)
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

pub const fn step(pos: Position, dir: Direction) -> Position {
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

const fn dir_from_signs(sx: i8, sy: i8) -> Direction {
    match (sx, sy) {
        (1, 0) => Direction::East,
        (-1, 0) => Direction::West,
        (0, 1) => Direction::South,
        (0, -1 | 0) => Direction::North,
        (1, -1) => Direction::NorthEast,
        (-1, -1) => Direction::NorthWest,
        (1, 1) => Direction::SouthEast,
        (-1, 1) => Direction::SouthWest,
        _ => Direction::North,
    }
}

/// How far a dino looks for food or a mate. Chebyshev rings, not a full scan.
pub const MAX_SEEK_RADIUS: usize = 24;

/// Visit cells in Chebyshev-distance order (closest first). `visit` returns
/// `true` to stop. Off-board cells are skipped.
pub fn visit_chebyshev_rings(
    origin: Position,
    max_radius: usize,
    width: usize,
    height: usize,
    mut visit: impl FnMut(Position) -> bool,
) {
    let ox = origin.x as isize;
    let oy = origin.y as isize;
    let w = width as isize;
    let h = height as isize;

    let mut push = |x: isize, y: isize| -> bool {
        if x < 0 || y < 0 || x >= w || y >= h {
            return false;
        }
        visit(Position {
            x: x as usize,
            y: y as usize,
            z: origin.z,
        })
    };

    if push(ox, oy) {
        return;
    }

    for r in 1..=max_radius as isize {
        for dx in -r..=r {
            if push(ox + dx, oy - r) {
                return;
            }
            if push(ox + dx, oy + r) {
                return;
            }
        }
        for dy in (-r + 1)..r {
            if push(ox - r, oy + dy) {
                return;
            }
            if push(ox + r, oy + dy) {
                return;
            }
        }
    }
}

/// First occupied cell in Chebyshev order that `pred` accepts.
pub fn find_nearest_on_grid(
    world: &WorldMap,
    origin: Position,
    max_radius: usize,
    mut pred: impl FnMut(Entity, Position) -> bool,
) -> Option<(Entity, Position)> {
    let (width, height) = world.dimensions();
    let mut found = None;
    visit_chebyshev_rings(origin, max_radius, width, height, |pos| {
        if let Some(entity) = world.entity_at(pos)
            && pred(entity, pos)
        {
            found = Some((entity, pos));
            return true;
        }
        false
    });
    found
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

    let mut candidates = [Direction::North; 3];
    let mut n = 0usize;
    if sx != 0 && sy != 0 {
        candidates[n] = dir_from_signs(sx, sy);
        n += 1;
    }
    if sx != 0 {
        candidates[n] = dir_from_signs(sx, 0);
        n += 1;
    }
    if sy != 0 {
        candidates[n] = dir_from_signs(0, sy);
        n += 1;
    }

    for dir in candidates.iter().take(n) {
        if neighbor_free(dino_pos, *dir, world) {
            return *dir;
        }
    }

    candidates[0]
}
