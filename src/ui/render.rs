use std::collections::HashMap;

use bevy::prelude::*;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType, Paragraph, Widget},
};

use crate::{
    app::{Performance, SystemPerformance},
    simulation::{Corpse, DinosaurStats, Egg, Plant},
    ui::TuiTerminal,
    world::{Board, Position, Renderable},
};

#[derive(Resource)]
pub struct Camera {
    x: usize,
    y: usize,
    percent: u16,
    pub last_view_w: usize,
    pub last_view_h: usize,
    pub is_dragging: bool,
    pub last_mouse_pos: Option<(u16, u16)>,
}

impl Camera {
    pub fn new(x: usize, y: usize, percent: u16) -> Self {
        Self {
            x,
            y,
            percent,
            last_view_w: 0,
            last_view_h: 0,
            is_dragging: false,
            last_mouse_pos: None,
        }
    }

    pub fn move_by(&mut self, dx: isize, dy: isize, board: &Board, view_w: usize, view_h: usize) {
        let max_x = board.width.saturating_sub(view_w);
        let max_y = board.height.saturating_sub(view_h);

        self.x = (self.x as isize + dx).clamp(0, max_x as isize) as usize;
        self.y = (self.y as isize + dy).clamp(0, max_y as isize) as usize;
    }
}

pub fn render(
    mut terminal: ResMut<TuiTerminal>,
    board: Res<Board>,
    entity_renderables: Query<(&Position, &Renderable)>,
    plants: Query<(), With<Plant>>,
    eggs: Query<(), With<Egg>>,
    corpses: Query<(), With<Corpse>>,
    entities: Query<()>,
    dinos: Query<&DinosaurStats>,
    performance: Res<Performance>,
    mut camera: ResMut<Camera>,
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

    let dinos_count = dinos.count();
    let plant_count = plants.count();
    let egg_count = eggs.count();
    let corpse_count = corpses.count();
    let entity_count = entities.count();
    let ticks_per_second = performance.ticks_per_second;

    terminal
        .0
        .draw(|frame| {
            let title = Line::from_iter([
                Span::from("Parque jurásico simulador").bold(),
                Span::from(" (Presioná 'q' o Ctrl + 'c' para salir)"),
            ]);

            // título arriba, resto abajo
            let root = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
            let [top, main] = frame.area().layout(&root);
            frame.render_widget(title.centered(), top);

            // "main" se parte en fila de arriba (board+stats) y fila de abajo (chart)
            let rows = Layout::vertical([Constraint::Fill(2), Constraint::Fill(1)])
                .spacing(1)
                .split(main);
            let board_row = rows[0];
            let chart_row = rows[1];

            let (view_w, view_h) = BoardWidget::viewport_size(board_row, camera.percent);
            camera.last_view_w = view_w;
            camera.last_view_h = view_h;
            camera.x = camera.x.min(board.width.saturating_sub(view_w));
            camera.y = camera.y.min(board.height.saturating_sub(view_h));

            let cols = Layout::horizontal([
                Constraint::Length((view_w * 2 + 2) as u16),
                Constraint::Min(20),
            ])
            .split(board_row);

            let board_widget = BoardWidget {
                percent: camera.percent,
                camera_x: camera.x,
                camera_y: camera.y,
                tiles: &visible,
            };
            frame.render_widget(board_widget, cols[0]);

            let stats = Paragraph::new(vec![
                Line::from(format!("Dinos:     {}", dinos_count)),
                Line::from(format!("Plants:    {}", plant_count)),
                Line::from(format!("Eggs:      {}", egg_count)),
                Line::from(format!("Corpses:   {}", corpse_count)),
                Line::from(format!("Entities:  {}", entity_count)),
                Line::from(format!("TPS:       {}", ticks_per_second)),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Stats"),
            );
            frame.render_widget(stats, cols[1]);

            render_chart(frame, chart_row, dinos);
        })
        .expect("failed to draw TUI");
}
struct BoardWidget<'a> {
    percent: u16,
    camera_x: usize,
    camera_y: usize,
    tiles: &'a HashMap<(usize, usize), (usize, &'a Renderable)>,
}

impl BoardWidget<'_> {
    fn viewport_size(area: Rect, percent: u16) -> (usize, usize) {
        let usable_width = area.width * percent / 100;
        let usable_height = area.height * percent / 100;

        let cols = ((usable_width.saturating_sub(2)) / 2) as usize;
        let rows = (usable_height.saturating_sub(2)) as usize;

        (cols, rows)
    }
}
impl Widget for BoardWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let (view_width, view_height) = Self::viewport_size(area, self.percent);

        let board_width = view_width as u16 * 2 + 2;
        let board_height = view_height as u16 + 2;

        let board_area = Rect {
            x: area.x,
            y: area.y,
            width: board_width.min(area.width),
            height: board_height.min(area.height),
        };

        let block = Block::default().borders(Borders::ALL);
        block.clone().render(board_area, buf);

        let inner = block.inner(board_area);

        for y in 0..view_height.min(inner.height as usize) {
            for x in 0..view_width.min(inner.width as usize / 2) {
                let world_x = self.camera_x + x;
                let world_y = self.camera_y + y;

                let (glyph, color) = self
                    .tiles
                    .get(&(world_x, world_y))
                    .map(|(_, r)| (r.glyph, r.color))
                    .unwrap_or(("  ", (0, 0, 0)));

                let cell_x = inner.x + (x * 2) as u16;
                let cell_y = inner.y + y as u16;

                buf.set_string(
                    cell_x,
                    cell_y,
                    glyph,
                    Style::default().fg(Color::Rgb(color.0, color.1, color.2)),
                );
            }
        }
    }
}

pub fn render_chart(frame: &mut Frame, area: Rect, dinos: Query<&DinosaurStats>) {
    let mut metabolisms: Vec<f64> = vec![];
    let mut resistances: Vec<f64> = vec![];
    for DinosaurStats {
        metabolism,
        starvation_resistance,
        ..
    } in dinos
    {
        metabolisms.push(*metabolism);
        resistances.push(*starvation_resistance);
    }

    let total_count = dinos.count();
    let metabolisms_data = get_data(metabolisms, total_count);
    let resistance_data = get_data(resistances, total_count);

    let dataset_metabolism = Dataset::default()
        .name("Metabolism")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Color::Red)
        .data(&metabolisms_data);

    let dataset_resistance = Dataset::default()
        .name("Resistance")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Color::Yellow)
        .data(&resistance_data);

    let x_axis = Axis::default()
        .bounds([0.0, 1.0])
        .labels(["0%", "50%", "100%"]);

    let half_count = total_count as u64 / 2;
    let half_count_str = &half_count.to_string();
    let total_count_str = &total_count.to_string();
    let y_axis = Axis::default()
        .title("Dinos".red())
        .bounds([0.0, 1.0])
        .labels(["0", &half_count_str, &total_count_str]);

    let chart = Chart::new(vec![dataset_metabolism, dataset_resistance])
        .x_axis(x_axis)
        .y_axis(y_axis);
    frame.render_widget(chart, area);
}

fn get_data(variable: Vec<f64>, total_count: usize) -> Vec<(f64, f64)> {
    let total_count = total_count as f64;
    let mut data: Vec<(f64, f64)> = vec![];
    let mut x: f64 = 0.0;
    for _ in 0..10 {
        let count = variable
            .iter()
            .filter(|m| **m >= x - 0.05 && **m < x + 0.05)
            .count();
        let count: f64 = count as f64;
        data.push((x, count / total_count));
        x += 0.1;
    }
    data
}
