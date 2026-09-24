use bevy::prelude::*;

use crate::{
    simulation::{
        Desire, Gender, Health, Maturity,
        components::{DinoOptions, DinoState, Herbivore, Hunger, Plant, Pregnant},
        movement::{MAX_SEEK_RADIUS, chebyshev, find_nearest_on_grid, neighborhood},
    },
    world::{Position, WorldMap},
};

#[allow(clippy::struct_excessive_bools)]
struct DinoView {
    entity: Entity,
    pos: Position,
    male: bool,
    pregnant: bool,
    hunger: f64,
    maturity: f64,
    health: f64,
    desire: f64,
    can_lay: bool,
    food_in_reach: bool,
}

pub fn dino_decision_system(
    mut dinos: Query<(
        Entity,
        &Position,
        &Hunger,
        &Maturity,
        &Health,
        &Desire,
        &Gender,
        Option<&Pregnant>,
        Option<&Herbivore>,
        &mut DinoState,
    )>,
    plants: Query<(), With<Plant>>,
    world: WorldMap,
) {
    let mut views = Vec::new();
    for (entity, pos, hunger, maturity, health, desire, gender, pregnant, herbivore, _) in &dinos {
        let can_lay = neighborhood(*pos)
            .into_iter()
            .skip(1)
            .any(|cell| world.is_free(cell));
        let food_in_reach = herbivore.is_some()
            && find_nearest_on_grid(&world, *pos, MAX_SEEK_RADIUS, |entity, _| {
                plants.get(entity).is_ok()
            })
            .is_some();
        views.push(DinoView {
            entity,
            pos: *pos,
            male: gender.0,
            pregnant: pregnant.is_some(),
            hunger: hunger.hunger(),
            maturity: maturity.maturity(),
            health: health.health(),
            desire: desire.desire(),
            can_lay,
            food_in_reach,
        });
    }

    for view in &views {
        let mate_in_reach = views.iter().any(|other| {
            other.entity != view.entity
                && other.male != view.male
                && chebyshev(&view.pos, &other.pos) <= MAX_SEEK_RADIUS
                && DinoState::would_mate(
                    other.pregnant,
                    other.hunger,
                    other.maturity,
                    other.health,
                    other.desire,
                    other.can_lay,
                    other.food_in_reach,
                )
        });
        let Ok((_, _, _, _, _, _, _, _, _, mut state)) = dinos.get_mut(view.entity) else {
            continue;
        };
        *state = DinoState::decide(
            view.pregnant,
            view.hunger,
            view.maturity,
            view.health,
            view.desire,
            DinoOptions {
                can_lay: view.can_lay,
                food_in_reach: view.food_in_reach,
                mate_in_reach,
            },
        );
    }
}
