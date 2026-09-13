use bevy::ecs::component::Component;

#[derive(Component)]
pub struct Egg {
    pub age: u32,
}

#[derive(Component)]
pub struct Health(pub f64);

impl Health {
    pub fn health(&self) -> f64 {
        self.0
    }
    pub fn decrease(&mut self, amount: f64) {
        if self.0 > 0.0 {
            self.0 -= amount;
        } else {
            self.0 = 0.0;
        }
    }
    pub fn increase(&mut self, amount: f64) {
        if self.0 < 1.0 {
            self.0 += amount;
        } else {
            self.0 = 1.0;
        }
    }
}
#[derive(Component, Copy, Clone, Debug)]
pub struct Desire(pub f64);

impl Desire {
    pub fn desire(&self) -> f64 {
        self.0
    }

    pub fn set_to_zero(&mut self) {
        self.0 = 0.0;
    }

    pub fn increase(&mut self, amount: f64) {
        if self.0 < 1.0 {
            self.0 += amount;
        } else {
            self.0 = 1.0;
        }
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

/// 0 = newborn, 1 = old.
#[derive(Component)]
pub struct Maturity(pub f64);

impl Maturity {
    pub fn maturity(&self) -> f64 {
        self.0
    }

    pub fn increase(&mut self, amount: f64) {
        if self.0 < 1.0 {
            self.0 += amount;
        } else {
            self.0 = 1.0;
        }
    }
}

#[derive(Component)]
pub struct Corpse {
    pub original_color: (u8, u8, u8),
}

#[derive(Component)]
pub struct Plant {
    pub health: u32,
    pub original_color: (u8, u8, u8),
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
    pub generation: u32, // Número de ancestros
    pub color: (u8, u8, u8),
    pub starvation_resistance: f64, // Cuánta hambre necesita para empezar a perder vida
    pub metabolism: f64,            // Cuánta hambre tiene, qué tan rapido madura, que tan rojo es.
}

#[derive(Component, Default, Eq, PartialEq, Debug, Clone, Copy)]
pub enum DinoState {
    #[default]
    Wandering,
    Healing,
    SeekingPartner,
    SeekingFood,
    LayingEgg,
    Dieing,
}

/// `true` = male, `false` = female. Females receive `Pregnant` on mating.
#[derive(Component, Clone, Copy, Debug)]
pub struct Gender(pub bool);

impl DinoState {
    /// Food always wins over mating; a pregnant dino only lays.
    pub fn decide(is_pregnant: bool, hunger: f64, maturity: f64, health: f64, desire: f64) -> Self {
        if maturity >= 1.0 || health <= 0.0 {
            DinoState::Dieing
        } else if is_pregnant {
            DinoState::LayingEgg
        } else if hunger > 0.5 {
            DinoState::SeekingFood
        } else if health < 1.0 {
            DinoState::Healing
        }else if desire >= 1.0 && maturity > 0.3 {
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
    pub degradation_threshold: f64,
    pub degradation: f64,
}
#[derive(Component)]
pub struct Grass;

impl Decay {
    pub fn increase(&mut self, amount: f64) {
        self.degradation = (self.degradation + amount).min(self.degradation_threshold);
    }
}
