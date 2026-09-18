use bevy::prelude::*;
use rand::{rng, seq::SliceRandom};

use crate::{
    simulation::{
        Desire,
        components::{DinoState, DinosaurStats, Direction, Gender, Pregnant},
        genetics::blend_dino_stats,
        movement::{
            MAX_SEEK_RADIUS, chebyshev, find_closest_tile, find_nearest_on_grid, step, try_move,
        },
        spawn::egg_bundle,
    },
    world::{Position, WorldMap},
};

pub fn reproduction_system(
    mut dinos: Query<(
        Entity,
        &mut Position,
        &DinosaurStats,
        &DinoState,
        &Gender,
        &mut Desire,
    )>,
    mut world_map: WorldMap,
    mut commands: Commands,
) {
    let seekers: Vec<Entity> = dinos
        .iter()
        .filter(|(_, _, _, state, _, _)| **state == DinoState::SeekingPartner)
        .map(|(entity, _, _, _, _, _)| entity)
        .collect();

    for origin_entity in seekers {
        let Ok((_, pos, origin_stats, _, gender, _)) = dinos.get(origin_entity) else {
            continue;
        };
        let origin_pos = *pos;
        let origin_gender = *gender;
        let origin_stats = *origin_stats;

        let Some((target_entity, target_pos)) =
            find_nearest_on_grid(&world_map, origin_pos, MAX_SEEK_RADIUS, |entity, _| {
                if entity == origin_entity {
                    return false;
                }
                let Ok((_, _, _, state, gender, _)) = dinos.get(entity) else {
                    return false;
                };
                *state == DinoState::SeekingPartner && gender.0 != origin_gender.0
            })
        else {
            continue;
        };

        if chebyshev(&origin_pos, &target_pos) <= 1 {
            if !origin_gender.0 {
                let Ok((_, _, target_stats, _, _, _)) = dinos.get(target_entity) else {
                    continue;
                };
                let son = blend_dino_stats(*target_stats, origin_stats);
                commands.entity(origin_entity).insert(Pregnant(son));
                if let Ok((_, _, _, _, _, mut desire)) = dinos.get_mut(origin_entity) {
                    desire.set_to_zero();
                }
                if let Ok((_, _, _, _, _, mut desire)) = dinos.get_mut(target_entity) {
                    desire.set_to_zero();
                }
            }
            continue;
        }

        let move_direction = find_closest_tile(&world_map, &origin_pos, &target_pos);
        if let Ok((_, mut pos, stats, _, _, _)) = dinos.get_mut(origin_entity) {
            try_move(
                &mut world_map,
                origin_entity,
                &mut pos,
                move_direction,
                stats.metabolism,
            );
        }
    }
}

pub fn increase_desire_system(query: Query<(&mut Desire, &DinoState)>) {
    for (mut desire, state) in query {
        if *state != DinoState::Wandering {
            // Healthy Wandering Dino starts feeling desire
            continue;
        }
        desire.increase(0.1);
    }
}

pub fn lay_eggs_system(
    query: Query<(Entity, &Position, &DinoState, &Pregnant)>,
    world: WorldMap,
    mut commands: Commands,
) {
    for (entity, pos, state, pregnant) in query {
        if *state != DinoState::LayingEgg {
            continue;
        }

        let mut spawn_pos = None;
        let mut directions = Direction::ALL;
        directions.shuffle(&mut rng());

        for dir in directions {
            let target = step(*pos, dir);
            if world.is_free(target) {
                spawn_pos = Some(target);
                break;
            }
        }

        if let Some(egg_pos) = spawn_pos {
            commands.spawn(egg_bundle(egg_pos.x, egg_pos.y, pregnant.0));
            commands.entity(entity).remove::<Pregnant>();
        }
    }
}
