use std::collections::HashMap;

use bevy::prelude::*;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Axis, Block, BorderType, Borders, Chart, Dataset, GraphType, LegendPosition, Paragraph,
        Tabs, Widget,
    },
};

use crate::{
    app::{Performance, SystemPerformance},
    simulation::{Corpse, DinosaurStats, Egg, Plant, PopulationHistory},
    ui::TuiTerminal,
    world::{Board, Position, Renderable},
};

#[derive(Resource)]
pub struct Camera {
    x: usize,
    y: usize,
    pub last_view_w: usize,
    pub last_view_h: usize,
    pub is_dragging: bool,
    pub last_mouse_pos: Option<(u16, u16)>,
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum SideTab {
    #[default]
    Stats,
    Charts,
    Tps,
}

impl SideTab {
    const COUNT: usize = 3;

    fn index(self) -> usize {
        match self {
            Self::Stats => 0,
            Self::Charts => 1,
            Self::Tps => 2,
        }
    }

    fn from_index(index: usize) -> Self {
        match index % Self::COUNT {
            1 => Self::Charts,
            2 => Self::Tps,
            _ => Self::Stats,
        }
    }

    pub fn next(self) -> Self {
        Self::from_index(self.index() + 1)
    }

    pub fn prev(self) -> Self {
        Self::from_index(self.index() + Self::COUNT - 1)
    }

    fn titles() -> [&'static str; 3] {
        ["Stats", "Charts", "TPS"]
    }
}

impl Camera {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
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
    history: Res<PopulationHistory>,
    side_tab: Res<SideTab>,
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
                Span::from("Jurassic Sandbox").bold(),
                Span::from("  q / Ctrl+C quit  ·  Tab switch panel  ·  arrows pan"),
            ]);

            let root = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .spacing(1);
            let [top, main] = frame.area().layout(&root);
            frame.render_widget(title.centered(), top);

            let cols = Layout::horizontal([Constraint::Fill(3), Constraint::Min(42)]).split(main);
            let board_area = cols[0];
            let side_area = cols[1];

            let (view_w, view_h) = BoardWidget::viewport_size(board_area);
            camera.last_view_w = view_w;
            camera.last_view_h = view_h;
            camera.x = camera.x.min(board.width.saturating_sub(view_w));
            camera.y = camera.y.min(board.height.saturating_sub(view_h));

            frame.render_widget(
                BoardWidget {
                    camera_x: camera.x,
                    camera_y: camera.y,
                    tiles: &visible,
                },
                board_area,
            );

            let side = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(side_area);
            let tabs = Tabs::new(SideTab::titles())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Inspect  [1-3]"),
                )
                .select(side_tab.index())
                .highlight_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )
                .divider(" │ ");
            frame.render_widget(tabs, side[0]);

            match *side_tab {
                SideTab::Stats => render_stats(
                    frame,
                    side[1],
                    dinos_count,
                    plant_count,
                    egg_count,
                    corpse_count,
                    entity_count,
                    ticks_per_second,
                ),
                SideTab::Charts => {
                    let chart_rows =
                        Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).split(side[1]);
                    render_population_chart(frame, chart_rows[0], &history);
                    render_chart(frame, chart_rows[1], dinos);
                }
                SideTab::Tps => render_tps_chart(frame, side[1], &history),
            }
        })
        .expect("failed to draw TUI");
}

fn render_stats(
    frame: &mut Frame,
    area: Rect,
    dinos_count: usize,
    plant_count: usize,
    egg_count: usize,
    corpse_count: usize,
    entity_count: usize,
    ticks_per_second: f32,
) {
    let stats = Paragraph::new(vec![
        Line::from(format!("Dinosaurs   {dinos_count}")),
        Line::from(format!("Plants      {plant_count}")),
        Line::from(format!("Eggs        {egg_count}")),
        Line::from(format!("Corpses     {corpse_count}")),
        Line::from(format!("Entities    {entity_count}")),
        Line::from(""),
        Line::from(format!("TPS         {ticks_per_second:.0}")),
        Line::from(""),
        Line::from("1 Stats  2 Charts  3 TPS").dim(),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Stats"),
    );
    frame.render_widget(stats, area);
}

struct BoardWidget<'a> {
    camera_x: usize,
    camera_y: usize,
    tiles: &'a HashMap<(usize, usize), (usize, &'a Renderable)>,
}

impl BoardWidget<'_> {
    fn viewport_size(area: Rect) -> (usize, usize) {
        let cols = ((area.width.saturating_sub(2)) / 2) as usize;
        let rows = (area.height.saturating_sub(2)) as usize;
        (cols, rows)
    }
}
impl Widget for BoardWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let (view_width, view_height) = Self::viewport_size(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("World");
        block.clone().render(area, buf);

        let inner = block.inner(area);

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

fn render_chart(frame: &mut Frame, area: Rect, dinos: Query<&DinosaurStats>) {
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
    if total_count == 0 {
        render_placeholder(frame, area, "Trait distribution", "No dinosaurs yet");
        return;
    }

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
        .title("Trait value")
        .bounds([0.0, 1.0])
        .labels(["0%", "50%", "100%"]);

    let half_count = total_count as u64 / 2;
    let half_count_str = half_count.to_string();
    let total_count_str = total_count.to_string();
    let y_axis = Axis::default()
        .title("Share")
        .bounds([0.0, 1.0])
        .labels(["0".to_string(), half_count_str, total_count_str]);

    let chart = Chart::new(vec![dataset_metabolism, dataset_resistance])
        .legend_position(Some(LegendPosition::TopRight))
        .x_axis(x_axis)
        .y_axis(y_axis)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Trait distribution"),
        );
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
fn render_placeholder(frame: &mut Frame, area: Rect, title: &str, message: &str) {
    frame.render_widget(
        Paragraph::new(message).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title.to_string()),
        ),
        area,
    );
}

fn render_history_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    y_title: &str,
    history: &PopulationHistory,
    series: &[(&str, Color, &[(f64, f64)])],
) {
    if series.iter().all(|(_, _, data)| data.is_empty()) {
        render_placeholder(frame, area, title, "No samples yet");
        return;
    }

    let min_x = 0.0;
    let max_x = history.elapsed.max(1.0);
    let max_y = series
        .iter()
        .flat_map(|(_, _, data)| data.iter().map(|(_, y)| *y))
        .fold(1.0, f64::max);

    let datasets: Vec<Dataset> = series
        .iter()
        .map(|(name, color, data)| {
            Dataset::default()
                .name(*name)
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(*color)
                .data(data)
        })
        .collect();

    let x_axis = Axis::default()
        .title("Time (s)")
        .bounds([min_x, max_x])
        .labels([
            "0".to_string(),
            format!("{:.0}", max_x / 2.0),
            format!("{:.0}", max_x),
        ]);

    let y_axis = Axis::default()
        .title(y_title.to_string())
        .bounds([0.0, max_y])
        .labels([
            "0".to_string(),
            format!("{:.0}", max_y / 2.0),
            format!("{:.0}", max_y),
        ]);

    let chart = Chart::new(datasets)
        .legend_position(Some(LegendPosition::TopRight))
        .x_axis(x_axis)
        .y_axis(y_axis)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title.to_string()),
        );
    frame.render_widget(chart, area);
}

fn render_population_chart(frame: &mut Frame, area: Rect, history: &PopulationHistory) {
    render_history_chart(
        frame,
        area,
        "Population history",
        "Count",
        history,
        &[
            ("Dinos", Color::Green, &history.dinos),
            ("Plants", Color::Cyan, &history.plants),
            ("Eggs", Color::Yellow, &history.eggs),
            ("Corpses", Color::Gray, &history.corpses),
        ],
    );
}

fn render_tps_chart(frame: &mut Frame, area: Rect, history: &PopulationHistory) {
    render_history_chart(
        frame,
        area,
        "Ticks per second",
        "TPS",
        history,
        &[("TPS", Color::Magenta, &history.tps)],
    );
}
