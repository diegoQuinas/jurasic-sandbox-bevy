use std::collections::HashMap;

use bevy::prelude::*;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::{
    Performance, SystemPerformance,
    board::{Board, Position, Renderable},
    creatures::components::{Corpse, Dinosaur, Egg, Genes, Plant},
    terminal::TuiTerminal,
};

pub fn render(
    mut terminal: ResMut<TuiTerminal>,
    board: Res<Board>,
    entity_renderables: Query<(&Position, &Renderable)>,
    plants: Query<(), With<Plant>>,
    eggs: Query<(), With<Egg>>,
    corpses: Query<(), With<Corpse>>,
    dinos: Query<(), With<Dinosaur>>,
    genes: Query<&Genes>,
    entities: Query<()>,
    performance: Res<Performance>,
    _systems_performance: Res<SystemPerformance>,
) {
    let mut visible: HashMap<(usize, usize), (usize, &Renderable)> = HashMap::new();
    for (position, renderable) in entity_renderables.iter() {
        visible
            .entry((position.x, position.y))
            .and_modify(|current| {
                if position.z > current.0 {
                    *current = (position.z, renderable)
                }
            })
            .or_insert((position.z, renderable));
    }

    let dino_count = dinos.count();
    let plant_count = plants.count();
    let egg_count = eggs.count();
    let corpse_count = corpses.count();
    let entity_count = entities.count();
    let _genes: Vec<Genes> = genes.iter().cloned().collect();
    let totals = dino_count + plant_count + egg_count + corpse_count;
    let ticks_per_second = performance.ticks_per_second;

    terminal
        .0
        .draw(|frame| {
            let area = frame.area();

            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length((board.width * 4 + 2) as u16),
                    Constraint::Min(20),
                ])
                .split(area);

            let board_widget = BoardWidget {
                width: board.width,
                height: board.height,
                tiles: &visible,
            };
            frame.render_widget(board_widget, cols[0]);

            let sidebar = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(10), Constraint::Min(3)])
                .split(cols[1]);

            let stats = Paragraph::new(vec![
                Line::from(format!("Dinos:     {}", dino_count)),
                Line::from(format!("Plants:    {}", plant_count)),
                Line::from(format!("Eggs:      {}", egg_count)),
                Line::from(format!("Corpses:   {}", corpse_count)),
                Line::from(format!("Total:     {}", totals)),
                Line::from(format!("Entities:  {}", entity_count)),
                Line::from(format!("TPS:       {}", ticks_per_second)),
                Line::from(format!("Genes:     {:?}", genes)),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Stats"),
            );
            frame.render_widget(stats, sidebar[0]);
        })
        .expect("failed to draw TUI");
}

struct BoardWidget<'a> {
    width: usize,
    height: usize,
    tiles: &'a HashMap<(usize, usize), (usize, &'a Renderable)>,
}

impl Widget for BoardWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let board_width = self.width as u16 * 2 + 2;
        let board_height = self.height as u16 + 2;

        let board_area = Rect {
            x: area.x,
            y: area.y,
            width: board_width.min(area.width),
            height: board_height.min(area.height),
        };

        let block = Block::default().borders(Borders::ALL);
        block.clone().render(board_area, buf);

        let inner = block.inner(area);

        for y in 0..self.height.min(inner.height as usize) {
            for x in 0..self.width.min(inner.width as usize / 2) {
                let (glyph, color) = self
                    .tiles
                    .get(&(x, y))
                    .map(|(_, r)| (r.glyph, r.color))
                    .unwrap_or(("  ", Color::Reset));

                let cell_x = inner.x + (x * 2) as u16;
                let cell_y = inner.y + y as u16;

                buf.set_string(cell_x, cell_y, glyph, Style::default().fg(color));
            }
        }
    }
}
