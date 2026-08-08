use bevy::{
    app::{Plugin, Startup, Update},
    ecs::schedule::IntoScheduleConfigs,
};

use crate::{
    StartupSet,
    creatures::{spawn::spawn_creatures, systems::*},
};

pub mod components;
pub mod spawn;
pub mod systems;

pub struct CreaturesPlugin;

impl Plugin for CreaturesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, spawn_creatures.in_set(StartupSet::Creatures))
            .add_systems(
                Update,
                (
                    mature_eggs_system,
                    wander_system,
                    reproduction_system,
                    hunger_system,
                    starving_system,
                    death_system,
                    decay_system,
                    spawn_random_plants,
                )
                    .chain(),
            );
    }
}
