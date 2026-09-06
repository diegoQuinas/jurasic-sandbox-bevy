use std::{fmt::format, vec};

use bevy::{ecs::{
    bundle::Bundle,
    system::{Commands, Query},
}};
use crossterm::style::{Color, Stylize};
use rand::RngExt;

use crate::{
    board::{Board, Position, Renderable},
    creatures::components::*,
};

pub fn spawn_creatures(board: Query<&Board>, mut commands: Commands) {
    let board = board.single().expect("Board not found");
    let (width, height) = (board.width, board.height);
    const FAMILY: [u32; 10] = [1, 2, 3, 4, 5 ,6,7,8,9,10];
    const PALETTE: [Color; 10] = [
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue,
                Color::Magenta,
                Color::Cyan,
                Color::White,
                Color::DarkRed,
                Color::DarkGreen,
                Color::DarkBlue,
            ];
    let mut randomfamilies: Vec<Genes> = Vec::new();
    
    for i   in FAMILY  {
        let color = PALETTE[(i as usize - 1) % PALETTE.len()];
        let random_family = Genes {
                starving_resistance: rand::rng().random_range(-5..=5),
                glyph: format!("{}","D ".with(color)),
                color,
                family: i,
        };
         randomfamilies.push(random_family);        
    }

    for y in 0..height {
        for x in 0..width {
            let options = rand::rng().random_range(1..=10);
 
            match options {
                1 => {
                    if(randomfamilies.len() == 0) {
                        break;
                    }
                    let random_index = rand::rng().random_range(0..randomfamilies.len());
                    let randomfamily = randomfamilies.swap_remove(random_index);
                    
                    commands.spawn(dinosaur_bundle(x, y, &randomfamily));

                    
                    
                }
                2..10 => {}
                _ => {
                    continue;
                }
            }
        }
    }
}

pub fn dinosaur_bundle(x: usize, y: usize, genes: &Genes) -> impl Bundle {
    let n = 15;
    let n = (n + genes.starving_resistance).clamp(10, 20) as u32;
    (
        Dinosaur {},
        Hungry {
            hunger: 0,
            starvation_threshold: n,
        },
        Health(10),
        Position { x, y },
        Renderable {
            glyph: genes.glyph.clone(),
        },
        Mortal {},
        Wanderer {},
        genes.clone(),
    )
}

pub fn egg_bundle(x: usize, y: usize, genes: Genes) -> impl Bundle {
    (
        Egg { age: 0 },
        Position { x, y },
        genes,
        Renderable {
            glyph: String::from("🥚"),
        },
    )
}

pub fn corpse_bundle(x: usize, y: usize) -> impl Bundle {
    (
        Corpse,
        Renderable {
            glyph: String::from("💀"),
        },
        Position { x, y },
        Decay {
            degradation_threshold: 10,
            degradation: 0,
        },
    )
}

pub fn plant_bundle(x: usize, y: usize) -> impl Bundle {
    (
        Plant,
        Position { x, y },
        Renderable {
            glyph: String::from("🌱"),
        },
        Decay {
            degradation: 0,
            degradation_threshold: 10,
        },
    )
}
