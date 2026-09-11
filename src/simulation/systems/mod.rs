mod ai;
mod food;
mod lifecycle;
mod reproduction;

pub use ai::dino_decision_system;
pub use food::{herbivore_eating_system, seek_herbivore_food_system, spawn_random_plants_system};
pub use lifecycle::{
    death_system, decay_system, degrade_corpose_color_system, degrade_plant_color_system,
    healing_system, hunger_system, mature_eggs_system, maturing_system, starving_system,
};
pub use reproduction::{increase_desire_system, lay_eggs_system, reproduction_system};
