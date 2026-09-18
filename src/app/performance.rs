use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Performance {
    pub ticks: u64,
    pub elapsed: f32,
    pub ticks_per_second: f32,
}

#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct SystemPerformance {
    pub eggs_mature: f64,
    pub wander: f64,
    pub reproduction: f64,
    pub hunger: f64,
    pub starving: f64,
    pub death: f64,
    pub spawn_plants: f64,
    pub decay: f64,
}

pub fn performance_system(time: Res<Time>, mut performance: ResMut<Performance>) {
    performance.ticks = performance.ticks.saturating_add(1);
    performance.elapsed += time.delta_secs();

    if performance.elapsed >= 1.0 {
        let ticks = u16::try_from(performance.ticks.min(u64::from(u16::MAX))).unwrap_or(u16::MAX);
        performance.ticks_per_second = f32::from(ticks) / performance.elapsed;
        performance.ticks = 0;
        performance.elapsed = 0.0;
    }
}
