use bevy::prelude::*;

use crate::simulation::components::{DinoState, DinosaurStats, Hunger, Pregnant};

pub fn dino_decision_system(
    mut dinos: Query<(&DinosaurStats, &Hunger, &mut DinoState, Option<&Pregnant>)>,
) {
    for (dino_stats, hunger, mut dino_state, pregnant) in &mut dinos {
        *dino_state = DinoState::decide(dino_stats, pregnant.is_some(), hunger.hunger());
    }
}
