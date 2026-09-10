use bevy::ecs::component::Component;

#[derive(Component)]
pub struct Egg {
    pub age: u32,
}

#[derive(Component)]
pub struct Health(pub f64);

impl Health {
    pub fn decrease(&mut self, amount: f64) {
        if self.0 > 0.0 {
            self.0 -= amount;
        } else {
            self.0 = 0.0;
        }
    }
    pub fn is_dead(&self) -> bool {
        self.0 <= 0.0
    }
}

/// 0 = full, 1 = starving.
#[derive(Component)]
pub struct Hunger(pub f64);

impl Hunger {
    pub fn hunger(&self) -> f64 {
        self.0
    }

    pub fn increase(&mut self, amount: f64) {
        if self.0 < 1.0 {
            self.0 += amount;
        } else {
            self.0 = 1.0;
        }
    }

    pub fn decrease(&mut self, amount: f64) {
        if self.0 > 0.0 {
            self.0 -= amount;
        }
    }
}

#[derive(Component)]
pub struct Mortal {}

#[derive(Component)]
pub struct Corpse;

#[derive(Component)]
pub struct Plant {
    pub health: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
        Direction::NorthEast,
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::SouthWest,
    ];
}

#[derive(Component, Clone, Copy, Debug)]
pub struct DinosaurStats {
    pub generation: u32,
    pub color: (u8, u8, u8),
    pub starvation_resistance: f64,
    pub reproduction_desire: f64,
}

#[derive(Component, Default, Eq, PartialEq, Debug, Clone, Copy)]
pub enum DinoState {
    #[default]
    Wandering,
    SeekingPartner,
    SeekingFood,
    LayingEgg,
}

/// `true` = male, `false` = female. Females receive `Pregnant` on mating.
#[derive(Component, Clone, Copy, Debug)]
pub struct Gender(pub bool);

impl DinoState {
    /// Food always wins over mating; a pregnant dino only lays.
    pub fn decide(stats: &DinosaurStats, is_pregnant: bool, hunger: f64) -> Self {
        if is_pregnant {
            DinoState::LayingEgg
        } else if hunger > 0.5 {
            DinoState::SeekingFood
        } else if stats.reproduction_desire > 0.3 {
            DinoState::SeekingPartner
        } else {
            DinoState::Wandering
        }
    }
}

#[derive(Component)]
pub struct Herbivore {}

#[derive(Component)]
pub struct Pregnant(pub DinosaurStats);

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
