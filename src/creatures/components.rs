use bevy::ecs::component::Component;

#[derive(Component)]
pub struct Egg {
    pub age: u32,
}

#[derive(Component)]
pub struct Dinosaur {}

#[derive(Component)]
pub struct Health(pub u32);

impl Health {
    pub fn decrease(&mut self, amount: u32) {
        self.0 = self.0.saturating_sub(amount);
    }
    pub fn is_dead(&self) -> bool {
        self.0 <= 0
    }
}

#[derive(Component)]
pub struct Mortal {}

#[derive(Component)]
pub struct Corpse;

#[derive(Component)]
pub struct Plant;

pub enum Direction {
    North,
    South,
    East,
    West,
}
#[derive(Component, Clone,Debug)]
pub struct Genes {
    pub starving_resistance: i32,
    pub glyph: String,
    pub color: crossterm::style::Color,
    pub family: u32,
}

#[derive(Component)]
pub struct Hungry {
    pub hunger: u32,
    pub starvation_threshold: u32,
}

impl Hungry {
    pub fn increase(&mut self, amount: u32) {
        self.hunger = self.hunger.saturating_add(amount)
    }
}

#[derive(Component)]
pub struct Starving;

#[derive(Component)]
pub struct Wanderer;

#[derive(Component)]
pub struct Decay {
    pub degradation_threshold: u32,
    pub degradation: u32,
}

impl Decay {
    pub fn increase(&mut self) {
        self.degradation = self.degradation.saturating_add(1);
    }
}
