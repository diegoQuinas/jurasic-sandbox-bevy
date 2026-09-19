use bevy::prelude::*;

use crate::{
    app::StartupSet,
    simulation::systems::{
        degrade_corpose_color_system, degrade_plant_color_system, increase_desire_system,
    },
};

use super::{
    spawn::spawn_creatures,
    systems::{
        PopulationHistory, SampleTimer, death_system, decay_system, dino_decision_system,
        healing_system, herbivore_eating_system, hunger_system, lay_eggs_system,
        mature_eggs_system, maturing_system, reproduction_system, sample_population_system,
        seek_herbivore_food_system, spawn_random_plants_system, starving_system, wander_system,
    },
};

pub struct CreaturesPlugin;

impl Plugin for CreaturesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<PopulationHistory>()
            .insert_resource(SampleTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
            .add_systems(Startup, spawn_creatures.in_set(StartupSet::Creatures))
            .add_systems(
                Update,
                (
                    (
                        dino_decision_system,
                        hunger_system,
                        increase_desire_system,
                        maturing_system,
                    ),
                    (
                        mature_eggs_system,
                        seek_herbivore_food_system,
                        wander_system,
                        reproduction_system,
                    ),
                    (
                        herbivore_eating_system,
                        lay_eggs_system,
                        starving_system,
                        healing_system,
                    ),
                    (
                        death_system,
                        decay_system,
                        degrade_corpose_color_system,
                        degrade_plant_color_system,
                        spawn_random_plants_system,
                    ),
                    sample_population_system,
                )
                    .chain(),
            );
    }
}
