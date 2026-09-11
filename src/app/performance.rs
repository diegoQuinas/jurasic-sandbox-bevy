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
    performance.ticks += 1;
    performance.elapsed += time.delta_secs();

    if performance.elapsed >= 1.0 {
        performance.ticks_per_second = performance.ticks as f32 / performance.elapsed;
        performance.ticks = 0;
        performance.elapsed = 0.0;
    }
}
