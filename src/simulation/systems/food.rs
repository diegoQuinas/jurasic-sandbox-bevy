use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use rand::{RngExt, rng};

use crate::{
    config::Config,
    simulation::{
        components::{DinoState, DinosaurStats, Herbivore, Hunger, Plant},
        movement::{
            MAX_SEEK_RADIUS, find_closest_tile, find_nearest_on_grid, neighborhood, try_move,
        },
        spawn::{ForestNoise, plant_bundle, trunk_bundle},
    },
    world::{Position, WorldMap},
};

pub fn seek_herbivore_food_system(
    mut dinos: Query<
        (Entity, &mut Position, &DinoState, &DinosaurStats),
        (With<Herbivore>, Without<Plant>),
    >,
    plants_query: Query<(), With<Plant>>,
    mut world_map: WorldMap,
) {
    let is_plant = |e: Entity| plants_query.get(e).is_ok();

    // First adjacent SeekingFood dino owns the plant.
    let mut reserved: HashSet<Entity> = HashSet::new();
    let mut plant_owner: HashMap<Entity, Entity> = HashMap::new();
    for (dino_e, dino_pos, state, _) in dinos.iter() {
        if *state != DinoState::SeekingFood {
            continue;
        }
        let adjacent_plant = neighborhood(*dino_pos)
            .into_iter()
            .find_map(|cell| world_map.entity_at(cell).filter(|e| is_plant(*e)));
        if let Some(plant_e) = adjacent_plant {
            plant_owner.entry(plant_e).or_insert(dino_e);
            reserved.insert(plant_e);
        }
    }

    for (dino_entity, mut dino_pos, dino_state, stats) in &mut dinos {
        if *dino_state != DinoState::SeekingFood {
            continue;
        }

        let eating_own_plant = neighborhood(*dino_pos).into_iter().any(|cell| {
            world_map
                .entity_at(cell)
                .is_some_and(|e| plant_owner.get(&e) == Some(&dino_entity))
        });
        if eating_own_plant {
            continue;
        }

        let mut fallback = None;
        let closest = find_nearest_on_grid(&world_map, *dino_pos, MAX_SEEK_RADIUS, |e, pos| {
            if !is_plant(e) {
                return false;
            }
            if reserved.contains(&e) {
                if fallback.is_none() {
                    fallback = Some((e, pos));
                }
                return false;
            }
            true
        });
        let Some((plant_entity, target_pos)) = closest.or(fallback) else {
            continue;
        };

        reserved.insert(plant_entity);

        let move_direction = find_closest_tile(&world_map, &dino_pos, &target_pos);
        try_move(
            &mut world_map,
            dino_entity,
            &mut dino_pos,
            move_direction,
            stats.metabolism,
        );
    }
}

pub fn herbivore_eating_system(
    mut commands: Commands,
    mut dinos: Query<(&Position, &mut Hunger, &DinoState), (With<Herbivore>, Without<Plant>)>,
    mut plants: Query<&mut Plant, Without<Herbivore>>,
    mut world: WorldMap,
) {
    // Two dinos must not finish the same plant in one tick.
    let mut eaten_plants = std::collections::HashSet::new();

    for (dino_pos, mut hungry, state) in &mut dinos {
        if *state != DinoState::SeekingFood {
            continue;
        }
        // A dino can only ever be adjacent to the 9 cells around it, so look
        // those up on the grid instead of scanning every plant.
        for cell in neighborhood(*dino_pos) {
            let Some(plant_entity) = world.entity_at(cell) else {
                continue;
            };
            if eaten_plants.contains(&plant_entity) {
                continue;
            }
            let Ok(mut plant) = plants.get_mut(plant_entity) else {
                continue;
            };

            eaten_plants.insert(plant_entity);
            if plant.health == 0 {
                commands.entity(plant_entity).despawn();
                commands.spawn(trunk_bundle(cell.x, cell.y));
                world.set_free(cell);
                hungry.decrease(1.0);
                break;
            }
            plant.health = plant.health.saturating_sub(1);
        }
    }
}

pub fn spawn_random_plants_system(
    mut commands: Commands,
    mut world: WorldMap,
    forest: Res<ForestNoise>,
    config: Res<Config>,
    trees: Query<(), With<Plant>>,
) {
    let count = trees.count();
    if count >= config.world.max_trees {
        return;
    }
    let mut rng = rng();
    let (width, height) = world.dimensions();
    if width == 0 || height == 0 {
        return;
    }

    // Bias samples toward groves: reject cells outside the noise field.
    let mut spawned = 0u8;
    for _ in 0..80 {
        if spawned >= 6 {
            break;
        }
        let x = rng.random_range(0..width);
        let y = rng.random_range(0..height);
        let density = forest.grove_density(x, y);
        if density <= 0.0 || !rng.random_bool(0.55f64.mul_add(density, 0.25)) {
            continue;
        }
        let pos = Position { x, y, z: 2 };
        if !world.is_free(pos) {
            continue;
        }
        let plant_entity = commands.spawn(plant_bundle(x, y)).id();
        world.set_occupied(plant_entity, pos);
        spawned += 1;
    }
}
