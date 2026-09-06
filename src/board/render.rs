use std::{
    collections::HashMap,
    io::{Write, stdout},
};

use bevy::prelude::*;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use crate::{
    Performance, SystemPerformance,
    board::{Board, Position, Renderable},
    creatures::components::{Corpse, Dinosaur, Egg, Genes, Plant},
};

pub fn render(
    board: Query<&Board>,
    entity_renderables: Query<(&Position, &Renderable)>,
    plants: Query<(), With<Plant>>,
    eggs: Query<(), With<Egg>>,
    corpses: Query<(), With<Corpse>>,
    dinos: Query<(), With<Dinosaur>>,
    genes: Query<&Genes, With<Genes>>,
    entities: Query<()>,
    performance: Res<Performance>,
    systems_performance: Res<SystemPerformance>,
) {
    let headless = false;
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0),).unwrap();
    if !headless {
        let board = board.single().expect("Can't find board");
        let visible_tiles: HashMap<(usize, usize), (usize, &String)> =
            entity_renderables.iter().fold(
                HashMap::<(usize, usize), (usize, &String)>::new(),
                |mut visible, (position, renderable)| {
                    visible
                        .entry((position.x, position.y))
                        .and_modify(|current| {
                            if position.z > current.0 {
                                *current = (position.z, &renderable.glyph)
                            }
                        })
                        .or_insert((position.z, &renderable.glyph));
                    visible
                },
            );

        let top_separator = {
            let mut s = String::with_capacity(board.width * 4 + 2);

            s.push('┏');

            for _ in 0..board.width - 1 {
                s.push_str("━━━━━");
            }
            s.push_str("━━━━┓");

            s.push('\n');
            s
        };
        let separator = {
            let mut s = String::with_capacity(board.width * 5 + 2);

            s.push('┃');

            for _ in 0..board.width - 1 {
                s.push_str("----┼");
            }
            s.push_str("----┃");

            s.push('\n');
            s
        };
        let bottom_separator = {
            let mut s = String::with_capacity(board.width * 5 + 2);

            s.push('┗');

            for _ in 0..board.width - 1 {
                s.push_str("━━━━━");
            }
            s.push_str("━━━━┛");

            s.push('\n');
            s
        };

        // Start rendering
        print!("{}", &top_separator);
        for y in 0..board.height {
            print!("┃ ");
            for x in 0..board.width {
                let glyph = visible_tiles
                    .get(&(x, y))
                    .map(|(_, glyph)| glyph.as_str())
                    .unwrap_or("  ");
                if x == board.width - 1 {
                    print!("{} ┃", glyph);
                } else {
                    print!("{} | ", glyph);
                }
            }
            if y == board.height - 1 {
                print!("\n{}", &bottom_separator)
            } else {
                print!("\n{}", &separator);
            }
        }
    }
    let dinos = dinos.count();
    let plants = plants.count();
    let eggs = eggs.count();
    let corpses = corpses.count();
    let genes = genes
        .iter()
        .map(|g: &Genes| g.clone())
        .collect::<Vec<Genes>>();
    let totals: usize = [dinos, plants, eggs, corpses].iter().sum();
    println!("Dinos: {}", dinos);
    println!("Plants: {}", plants);
    println!("Eggs: {}", eggs);
    println!("Corpses: {}", corpses);
    println!("Total : {}", totals);
    println!("Entities: {}", entities.count());
    println!("Ticks per second: {}", performance.ticks_per_second);
    /*println!("=== Performance ===");
    println!("eggs_mature: {:.4}", systems_performance.eggs_mature);
    println!("wander: {:.4}", systems_performance.wander);
    println!("reproduction: {:.4}", systems_performance.reproduction);
    println!("hunger: {:.4}", systems_performance.hunger);
    println!("starving: {:.4}", systems_performance.starving);
    println!("spawn_plants: {:.4}", systems_performance.spawn_plants);
    println!("decay: {:.4}", systems_performance.decay);*/
    for gene in genes {
        println!(
            "Color {:?} resistance {:?} ",
            gene.color, gene.starving_resistance
        );
    }
    stdout().flush().unwrap();
}
