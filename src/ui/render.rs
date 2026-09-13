use std::collections::HashMap;

use bevy::prelude::*;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Position as CellPos, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Axis, Block, BorderType, Borders, Chart, Dataset, GraphType, LegendPosition, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs, Widget,
    },
};

use crate::{
    app::{Performance, SystemPerformance},
    simulation::{
        Corpse, Desire, DinosaurStats, Egg, Health, Hunger, Maturity, Plant, PopulationHistory,
    },
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

pub const SIDEBAR_MIN_WIDTH: u16 = 28;
const SIDEBAR_DEFAULT_WIDTH: u16 = 42;
const CHART_MIN_HEIGHT: u16 = 14;
const PICKER_HEIGHT: u16 = 6;

const COUNT_SERIES: [(&str, Color); 4] = [
    ("Dinos", Color::Green),
    ("Plants", Color::Cyan),
    ("Eggs", Color::Yellow),
    ("Corpses", Color::Gray),
];

const TRAIT_SERIES: [(&str, Color); 6] = [
    ("Metab", Color::Red),
    ("Resist", Color::Yellow),
    ("Hunger", Color::LightRed),
    ("Health", Color::LightGreen),
    ("Mature", Color::Blue),
    ("Desire", Color::Magenta),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChartHit {
    Count(usize),
    Trait(usize),
}

#[derive(Resource)]
pub struct ChartSelection {
    pub counts: [bool; 4],
    pub traits: [bool; 6],
}

impl Default for ChartSelection {
    fn default() -> Self {
        Self {
            counts: [true, true, true, true],
            traits: [true, true, false, false, false, false],
        }
    }
}

impl ChartSelection {
    pub fn toggle(&mut self, hit: ChartHit) {
        match hit {
            ChartHit::Count(i) => {
                if let Some(flag) = self.counts.get_mut(i) {
                    *flag = !*flag;
                }
            }
            ChartHit::Trait(i) => {
                if let Some(flag) = self.traits.get_mut(i) {
                    *flag = !*flag;
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct Sidebar {
    pub width: u16,
    pub scroll: u16,
    pub resizing: bool,
    pub last_tabs: Rect,
    pub last_content: Rect,
    pub last_side: Rect,
    pub last_chart_hits: Vec<(Rect, ChartHit)>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            width: SIDEBAR_DEFAULT_WIDTH,
            scroll: 0,
            resizing: false,
            last_tabs: Rect::default(),
            last_content: Rect::default(),
            last_side: Rect::default(),
            last_chart_hits: Vec::new(),
        }
    }
}

impl Sidebar {
    pub fn tab_at(&self, column: u16, row: u16) -> Option<SideTab> {
        if !self.last_tabs.contains(CellPos::new(column, row)) {
            return None;
        }
        // Titles are packed left: " Stats │ Charts │ TPS " (pad 1, divider " │ ").
        let inner = self.last_tabs.inner(Margin::new(1, 1));
        if inner.width == 0 || column < inner.x {
            return None;
        }
        const PAD: u16 = 1;
        const DIVIDER_WIDTH: u16 = 3;
        let titles = SideTab::titles();
        let mut x = inner.x;
        for (i, title) in titles.iter().enumerate() {
            let tab_w = PAD + title.len() as u16 + PAD;
            let last = i + 1 == titles.len();
            let span = if last { tab_w } else { tab_w + DIVIDER_WIDTH };
            let end = (x + span).min(inner.x + inner.width);
            if column >= x && column < end {
                return Some(SideTab::from_index(i));
            }
            x = end;
        }
        None
    }

    pub fn chart_hit_at(&self, column: u16, row: u16) -> Option<ChartHit> {
        let pos = CellPos::new(column, row);
        self.last_chart_hits
            .iter()
            .find(|(rect, _)| rect.contains(pos))
            .map(|(_, hit)| *hit)
    }

    pub fn is_resize_handle(&self, column: u16, row: u16) -> bool {
        if self.last_side.height == 0 {
            return false;
        }
        let y_ok = row >= self.last_side.y && row < self.last_side.y + self.last_side.height;
        y_ok && (column == self.last_side.x || column + 1 == self.last_side.x)
    }

    pub fn clamp_width(&self, terminal_width: u16) -> u16 {
        let max = terminal_width.saturating_sub(16).max(SIDEBAR_MIN_WIDTH);
        self.width.clamp(SIDEBAR_MIN_WIDTH, max)
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
    dinos: Query<(&DinosaurStats, &Hunger, &Health, &Maturity, &Desire)>,
    performance: Res<Performance>,
    mut camera: ResMut<Camera>,
    history: Res<PopulationHistory>,
    side_tab: Res<SideTab>,
    mut sidebar: ResMut<Sidebar>,
    charts: Res<ChartSelection>,
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
                Span::from(
                    "  q / Ctrl+C quit  ·  click tabs  ·  drag sidebar  ·  Tab  ·  arrows pan",
                ),
            ]);

            let root = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .spacing(1);
            let [top, main] = frame.area().layout(&root);
            frame.render_widget(title.centered(), top);

            let side_width = sidebar.clamp_width(main.width);
            sidebar.width = side_width;
            let cols = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(side_width),
            ])
            .split(main);
            let board_area = cols[0];
            let side_area = cols[1];
            sidebar.last_side = side_area;

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
            sidebar.last_tabs = side[0];
            sidebar.last_content = side[1];
            let tabs = Tabs::new(SideTab::titles())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Inspect  click / [1-3]  ·  drag │"),
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
                    render_scrollable_charts(
                        frame,
                        side[1],
                        &history,
                        dinos,
                        &mut sidebar,
                        &charts,
                    );
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

fn clip_rect(area: Rect, parent: Rect) -> Option<Rect> {
    let clipped = area.intersection(parent);
    if clipped.width == 0 || clipped.height == 0 {
        None
    } else {
        Some(clipped)
    }
}

fn render_scrollable_charts(
    frame: &mut Frame,
    area: Rect,
    history: &PopulationHistory,
    dinos: Query<(&DinosaurStats, &Hunger, &Health, &Maturity, &Desire)>,
    sidebar: &mut Sidebar,
    charts: &ChartSelection,
) {
    sidebar.last_chart_hits.clear();
    let picker_h = PICKER_HEIGHT.min(area.height);
    let body = Layout::vertical([Constraint::Length(picker_h), Constraint::Fill(1)]).split(area);
    render_series_picker(frame, body[0], charts, sidebar);

    let chart_area = body[1];
    let chart_count = 2u16;
    let min_total = CHART_MIN_HEIGHT.saturating_mul(chart_count);

    if chart_area.height >= min_total {
        sidebar.scroll = 0;
        let chunks = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).split(chart_area);
        render_population_chart(frame, chunks[0], history, charts);
        render_chart(frame, chunks[1], dinos, charts);
        return;
    }

    let max_scroll = min_total.saturating_sub(chart_area.height);
    sidebar.scroll = sidebar.scroll.min(max_scroll);

    let slot = |index: u16| {
        let y = chart_area
            .y
            .saturating_add(index * CHART_MIN_HEIGHT)
            .saturating_sub(sidebar.scroll);
        Rect {
            x: chart_area.x,
            y,
            width: chart_area.width.saturating_sub(1),
            height: CHART_MIN_HEIGHT,
        }
    };
    if let Some(visible) = clip_rect(slot(0), chart_area) {
        render_population_chart(frame, visible, history, charts);
    }
    if let Some(visible) = clip_rect(slot(1), chart_area) {
        render_chart(frame, visible, dinos, charts);
    }

    if max_scroll > 0 {
        let mut state = ScrollbarState::new(max_scroll as usize).position(sidebar.scroll as usize);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼")),
            chart_area,
            &mut state,
        );
    }
}

fn chip_style(on: bool, color: Color) -> Style {
    if on {
        Style::default().fg(color).add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn render_chip_row(
    x0: u16,
    y: u16,
    max_x: u16,
    items: &[(&str, Color)],
    enabled: &[bool],
    make_hit: impl Fn(usize) -> ChartHit,
    hits: &mut Vec<(Rect, ChartHit)>,
) -> Line<'static> {
    let mut spans = Vec::new();
    let mut x = x0;
    for (i, (name, color)) in items.iter().enumerate() {
        let on = enabled.get(i).copied().unwrap_or(false);
        let label = format!(" {name} ");
        let w = label.len() as u16;
        if x + w > max_x {
            break;
        }
        hits.push((
            Rect {
                x,
                y,
                width: w,
                height: 1,
            },
            make_hit(i),
        ));
        spans.push(Span::styled(label, chip_style(on, *color)));
        spans.push(Span::raw(" "));
        x = x.saturating_add(w + 1);
    }
    Line::from(spans)
}

fn render_series_picker(
    frame: &mut Frame,
    area: Rect,
    charts: &ChartSelection,
    sidebar: &mut Sidebar,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Series  click to toggle  ·  counts overlay  ·  traits overlay");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let counts = render_chip_row(
        inner.x,
        inner.y,
        inner.x + inner.width,
        &COUNT_SERIES,
        &charts.counts,
        ChartHit::Count,
        &mut sidebar.last_chart_hits,
    );
    let traits = if inner.height >= 2 {
        render_chip_row(
            inner.x,
            inner.y.saturating_add(1),
            inner.x + inner.width,
            &TRAIT_SERIES,
            &charts.traits,
            ChartHit::Trait,
            &mut sidebar.last_chart_hits,
        )
    } else {
        Line::from("")
    };

    frame.render_widget(Paragraph::new(vec![counts, traits]), inner);
}

fn render_chart(
    frame: &mut Frame,
    area: Rect,
    dinos: Query<(&DinosaurStats, &Hunger, &Health, &Maturity, &Desire)>,
    charts: &ChartSelection,
) {
    let mut buckets: [Vec<f64>; 6] = Default::default();
    let mut total_count = 0usize;
    for (stats, hunger, health, maturity, desire) in dinos {
        total_count += 1;
        buckets[0].push(stats.metabolism);
        buckets[1].push(stats.starvation_resistance);
        buckets[2].push(hunger.hunger());
        buckets[3].push(health.health());
        buckets[4].push(maturity.maturity());
        buckets[5].push(desire.desire());
    }

    if total_count == 0 {
        render_placeholder(frame, area, "Trait distribution", "No dinosaurs yet");
        return;
    }

    if !charts.traits.iter().any(|on| *on) {
        render_placeholder(
            frame,
            area,
            "Trait distribution",
            "Select one or more traits above",
        );
        return;
    }

    let plotted: Vec<(usize, Vec<(f64, f64)>)> = charts
        .traits
        .iter()
        .enumerate()
        .filter(|(_, on)| **on)
        .map(|(i, _)| (i, get_data(std::mem::take(&mut buckets[i]), total_count)))
        .collect();

    let datasets: Vec<Dataset> = plotted
        .iter()
        .map(|(i, data)| {
            let (name, color) = TRAIT_SERIES[*i];
            Dataset::default()
                .name(name)
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(color)
                .data(data)
        })
        .collect();

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

    let chart = Chart::new(datasets)
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

fn render_population_chart(
    frame: &mut Frame,
    area: Rect,
    history: &PopulationHistory,
    charts: &ChartSelection,
) {
    if !charts.counts.iter().any(|on| *on) {
        render_placeholder(
            frame,
            area,
            "Population history",
            "Select one or more counts above",
        );
        return;
    }

    let all = [
        ("Dinos", Color::Green, history.dinos.as_slice()),
        ("Plants", Color::Cyan, history.plants.as_slice()),
        ("Eggs", Color::Yellow, history.eggs.as_slice()),
        ("Corpses", Color::Gray, history.corpses.as_slice()),
    ];
    let series: Vec<(&str, Color, &[(f64, f64)])> = all
        .into_iter()
        .enumerate()
        .filter(|(i, _)| charts.counts[*i])
        .map(|(_, s)| s)
        .collect();
    render_history_chart(frame, area, "Population history", "Count", history, &series);
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
