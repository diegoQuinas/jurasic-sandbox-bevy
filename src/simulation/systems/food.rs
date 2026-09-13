use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use rand::{RngExt, rng};

use crate::{
    simulation::{
        components::{DinoState, Herbivore, Hunger, Plant},
        movement::{chebyshev, find_closest_tile, neighborhood, step},
        spawn::{grass_bundle, plant_bundle},
    },
    world::{Position, WorldMap},
};

pub fn seek_herbivore_food_system(
    mut dinos: Query<(Entity, &mut Position, &DinoState), (With<Herbivore>, Without<Plant>)>,
    plants_query: Query<(Entity, &Position), (With<Plant>, Without<Herbivore>)>,
    mut world_map: WorldMap,
) {
    let plants: Vec<(Entity, Position)> = plants_query.iter().map(|(e, p)| (e, *p)).collect();

    // First adjacent dino owns the plant; everyone else must pick another.
    // Adjacency only cares about the 9 cells around the dino, so look them up
    // on the grid instead of scanning every plant.
    let mut reserved: HashSet<Entity> = HashSet::new();
    let mut plant_owner: HashMap<Entity, Entity> = HashMap::new();
    for (dino_e, dino_pos, _) in dinos.iter() {
        let adjacent_plant = neighborhood(*dino_pos)
            .into_iter()
            .find_map(|cell| world_map.entity_at(cell).filter(|e| plants_query.get(*e).is_ok()));
        if let Some(plant_e) = adjacent_plant {
            plant_owner.entry(plant_e).or_insert(dino_e);
            reserved.insert(plant_e);
        }
    }

    for (dino_entity, mut dino_pos, dino_state) in &mut dinos {
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

        let closest_free = plants
            .iter()
            .filter(|(plant_e, _)| !reserved.contains(plant_e))
            .min_by_key(|(_, pos)| chebyshev(&dino_pos, pos))
            .copied();
        let closest_any = plants
            .iter()
            .min_by_key(|(_, pos)| chebyshev(&dino_pos, pos))
            .copied();
        let Some((plant_entity, target_pos)) = closest_free.or(closest_any) else {
            continue;
        };

        reserved.insert(plant_entity);

        let move_direction = find_closest_tile(&world_map, &dino_pos, &target_pos);
        let target = step(*dino_pos, move_direction);

        if target != *dino_pos && world_map.is_free(target) {
            world_map.move_entity(dino_entity, *dino_pos, target);
            *dino_pos = target;
        }
    }
}

pub fn herbivore_eating_system(
    mut commands: Commands,
    mut dinos: Query<(&Position, &mut Hunger), (With<Herbivore>, Without<Plant>)>,
    mut plants: Query<&mut Plant, Without<Herbivore>>,
    mut world: WorldMap,
) {
    // Two dinos must not finish the same plant in one tick.
    let mut eaten_plants = std::collections::HashSet::new();

    for (dino_pos, mut hungry) in &mut dinos {
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
                world.set_free(cell);
                hungry.decrease(1.0);
                break;
            } else {
                plant.health = plant.health.saturating_sub(1);
            }
        }
    }
}

pub fn spawn_random_plants_system(mut commands: Commands, mut world: WorldMap) {
    let mut rng = rng();
    let (width, height) = world.dimensions();
    for _ in 0..5 {
        let x = rng.random_range(0..width);
        let y = rng.random_range(0..height);
        let pos = Position { x, y, z: 2 };

        if !world.is_free(pos) {
            return;
        }
        let is_plant = rng.random_bool(0.1);

        if is_plant {
            let plant_entity = commands.spawn(plant_bundle(x, y)).id();
            world.set_occupied(plant_entity, pos);
        } else {
            //let _grass_entity = commands.spawn(grass_bundle(x, y));
        }
    }
}
