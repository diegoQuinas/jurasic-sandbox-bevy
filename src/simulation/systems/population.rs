use bevy::prelude::*;

use crate::{
    app::Performance,
    simulation::{Corpse, DinosaurStats, Egg, Plant},
};

#[derive(Resource, Default)]
pub struct PopulationHistory {
    pub elapsed: f64,
    pub dinos: Vec<(f64, f64)>,
    pub plants: Vec<(f64, f64)>,
    pub eggs: Vec<(f64, f64)>,
    pub corpses: Vec<(f64, f64)>,
    pub tps: Vec<(f64, f64)>,
}
#[derive(Resource)]
pub struct SampleTimer(pub Timer);

pub fn sample_population_system(
    time: Res<Time>,
    mut timer: ResMut<SampleTimer>,
    dinos: Query<(), With<DinosaurStats>>,
    plants: Query<(), With<Plant>>,
    eggs: Query<(), With<Egg>>,
    corpses: Query<(), With<Corpse>>,
    performance: Res<Performance>,
    mut history: ResMut<PopulationHistory>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    history.elapsed += timer.0.duration().as_secs_f64();
    let t = history.elapsed;

    push_sample(&mut history.dinos, (t, dinos.iter().count() as f64));
    push_sample(&mut history.plants, (t, plants.iter().count() as f64));
    push_sample(&mut history.eggs, (t, eggs.iter().count() as f64));
    push_sample(&mut history.corpses, (t, corpses.iter().count() as f64));
    push_sample(&mut history.tps, (t, performance.ticks_per_second as f64));
}

const MAX_HISTORY: usize = 360;

fn push_sample(series: &mut Vec<(f64, f64)>, point: (f64, f64)) {
    series.push(point);
    if series.len() > MAX_HISTORY * 2 {
        *series = series.iter().step_by(2).copied().collect();
    }
}
