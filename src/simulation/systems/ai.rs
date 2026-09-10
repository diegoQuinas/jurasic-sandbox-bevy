use bevy::prelude::*;

use crate::simulation::{
    Desire, Health, Maturity,
    components::{DinoState, Hunger, Pregnant},
};

pub fn dino_decision_system(
    mut dinos: Query<(
        &Hunger,
        &Maturity,
        &Health,
        &mut DinoState,
        Option<&Pregnant>,
        &Desire,
    )>,
) {
    for (hunger, maturity, health, mut dino_state, pregnant, desire) in &mut dinos {
        *dino_state = DinoState::decide(
            pregnant.is_some(),
            hunger.hunger(),
            maturity.maturity(),
            health.health(),
            desire.desire(),
        );
    }
}
