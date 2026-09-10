use bevy::prelude::*;
use rand::{rng, seq::SliceRandom};

use crate::{
    simulation::{
        components::{DinoState, DinosaurStats, Direction, Gender, Pregnant},
        genetics::blend_dino_stats,
        movement::{find_closest_target_reproduction, find_closest_tile, manhattan, step},
        spawn::egg_bundle,
    },
    world::{Position, WorldMap},
};

pub fn reproduction_system(
    mut origin: Query<(Entity, &mut Position, &DinosaurStats, &DinoState, &Gender)>,
    mut world_map: WorldMap,
    mut commands: Commands,
) {
    // Snapshot first: we cannot hold `&Position` and `&mut Position` on the same
    // dino population in two queries (Bevy B0001).
    let snapshots: Vec<_> = origin
        .iter()
        .map(|(entity, pos, stats, state, gender)| (entity, *pos, *stats, *state, *gender))
        .collect();

    for (origin_entity, mut pos, origin_stats, state, gender) in &mut origin {
        if *state != DinoState::SeekingPartner {
            continue;
        }
        let Some((_target_entity, target_pos, target_stats)) =
            find_closest_target_reproduction(&pos, gender, &snapshots)
        else {
            continue;
        };

        if manhattan(&pos, &target_pos) <= 1 {
            if !gender.0 {
                let son = blend_dino_stats(&target_stats, origin_stats);
                commands.entity(origin_entity).insert(Pregnant(son));
            }
            continue;
        }

        let move_direction = find_closest_tile(&world_map, &pos, &target_pos);
        let target = step(*pos, move_direction);

        if target != *pos && world_map.is_free(target) {
            world_map.move_entity(origin_entity, *pos, target);
            *pos = target;
        }
    }
}

pub fn lay_eggs_system(
    query: Query<(Entity, &Position, &DinoState, &Pregnant)>,
    mut world: WorldMap,
    mut commands: Commands,
) {
    for (entity, pos, state, pregnant) in query {
        if *state != DinoState::LayingEgg {
            continue;
        }

        let mut spawn_pos = None;
        let mut directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];
        directions.shuffle(&mut rng());

        for dir in directions {
            let target = step(*pos, dir);
            if world.is_free(target) {
                spawn_pos = Some(target);
                break;
            }
        }

        if let Some(egg_pos) = spawn_pos {
            let egg_entity = commands
                .spawn(egg_bundle(egg_pos.x, egg_pos.y, pregnant.0))
                .id();
            world.set_occupied(egg_entity, egg_pos);
            commands.entity(entity).remove::<Pregnant>();
        }
    }
}
