use bevy::prelude::*;
use rand::{RngExt, rng};

use crate::{
    simulation::{
        Corpse, DinoState, Maturity,
        components::{Decay, DinosaurStats, Egg, Health, Hunger, Mortal},
        spawn::{corpse_bundle, dinosaur_bundle},
    },
    world::{Occupancy, Position, Renderable, WorldMap},
};

pub fn maturing_system(query: Query<(&mut Maturity, &DinosaurStats)>) {
    for (mut maturity, stats) in query {
        let increasement = stats.metabolism * 0.001;
        maturity.increase(increasement);
    }
}

pub fn mature_eggs_system(
    mut commands: Commands,
    query: Query<(Entity, &mut Egg, &Position, &DinosaurStats)>,
    mut world: WorldMap,
) {
    for (entity, mut egg, pos, genes) in query {
        if egg.age < 90 {
            egg.age = egg.age.saturating_add(1);
        } else {
            if world.is_free(*pos) {
                let dino = commands.spawn(dinosaur_bundle(pos.x, pos.y, genes)).id();
                world.set_occupied(dino, *pos);
                commands.entity(entity).despawn();
            } else {
                continue;
            }
        }
    }
}

pub fn hunger_system(query: Query<(&mut Hunger, &DinosaurStats)>) {
    for (mut hunger, stats) in query {
        let increasement = stats.metabolism * 0.005;
        hunger.increase(increasement);
    }
}

pub fn starving_system(query: Query<(&Hunger, &DinosaurStats, &mut Health)>) {
    for (hunger, dino_stats, mut health) in query {
        if hunger.hunger() > dino_stats.starvation_resistance {
            health.decrease(0.01);
        }
    }
}

pub fn death_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &DinoState), With<Mortal>>,
    mut world: WorldMap,
) {
    for (e, p, s) in query {
        if *s != DinoState::Dieing {
            continue;
        } else {
            commands.spawn(corpse_bundle(p.x, p.y));
            world.set_free(*p);
            commands.entity(e).despawn();
        }
    }
}

pub fn healing_system(query: Query<(&mut Health, &DinoState)>) {
    for (mut health, state) in query {
        if *state != DinoState::Healing {
            continue;
        } else {
            health.increase(0.01);
        }
    }
}

pub fn degrade_corpose_color_system(query: Query<(&Decay, &mut Renderable), With<Corpse>>) {
    for (decay, mut renderable) in query {
        let degradation = decay.degradation;

        let color = if degradation < 0.3 {
            let t = degradation / 0.3;

            // Rojo -> verde podrido
            let r = 180.0 + (70.0 - 180.0) * t;
            let g = 30.0 + (90.0 - 30.0) * t;
            let b = 25.0 + (35.0 - 25.0) * t;

            (r as u8, g as u8, b as u8)
        } else {
            let t = (degradation - 0.3) / 0.7;

            // Verde podrido -> gris hueso
            let r = 70.0 + (145.0 - 70.0) * t;
            let g = 90.0 + (140.0 - 90.0) * t;
            let b = 35.0 + (120.0 - 35.0) * t;

            (r as u8, g as u8, b as u8)
        };

        renderable.color = color;
    }
}

pub fn degrade_plant_color_system(query: Query<(&Decay, &mut Renderable), With<Corpse>>) {
    for (decay, mut renderable) in query {
        let original = renderable.color;
        let degradation = decay.degradation;

        let degradation = degradation.clamp(0.0, 1.0);

        let target = (200.0, 180.0, 30.0);

        let r = original.0 as f64 + (target.0 - original.0 as f64) * degradation;
        let g = original.1 as f64 + (target.1 - original.1 as f64) * degradation;
        let b = original.2 as f64 + (target.2 - original.2 as f64) * degradation;

        let color = (r as u8, g as u8, b as u8);

        renderable.color = color;
    }
}
pub fn decay_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &mut Decay)>,
    mut occupancy: ResMut<Occupancy>,
) {
    let mut rng = rng();
    for (e, p, mut d) in query {
        let random = rng.random_range(0.001..=0.010);
        d.increase(random);

        if d.degradation >= d.degradation_threshold {
            occupancy.set(*p, None);
            commands.entity(e).despawn();
        }
    }
}
