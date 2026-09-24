use bevy::ecs::component::Component;
use rand::{RngExt, rng};

#[derive(Component)]
pub struct Egg {
    pub age: u32,
}

#[derive(Component)]
pub struct Health(pub f64);

impl Health {
    pub const fn health(&self) -> f64 {
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
    pub const fn desire(self) -> f64 {
        self.0
    }

    pub const fn set_to_zero(&mut self) {
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
    pub const fn hunger(&self) -> f64 {
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
    pub const fn maturity(&self) -> f64 {
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
    pub const ALL: [Self; 8] = [
        Self::North,
        Self::South,
        Self::East,
        Self::West,
        Self::NorthEast,
        Self::NorthWest,
        Self::SouthEast,
        Self::SouthWest,
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
    Wait,
    Wandering,
    Healing,
    SeekingPartner,
    SeekingFood,
    LayingEgg,
    Dieing,
}

/// What this dino can actually make progress on this tick.
#[derive(Clone, Copy, Debug, Default)]
pub struct DinoOptions {
    /// A free neighboring cell exists to drop an egg.
    pub can_lay: bool,
    /// A plant is inside the seek radius.
    pub food_in_reach: bool,
    /// An opposite-sex adult who would also mate is inside the seek radius.
    pub mate_in_reach: bool,
}

impl DinoState {
    /// Needs stay ordered (death, egg, food, heal, mate, wander), but a need
    /// that cannot be acted on is skipped so the tick is not spent waiting.
    pub fn decide(
        is_pregnant: bool,
        hunger: f64,
        maturity: f64,
        health: f64,
        desire: f64,
        options: DinoOptions,
    ) -> Self {
        if maturity >= 1.0 || health <= 0.0 {
            return Self::Dieing;
        }
        if is_pregnant && options.can_lay {
            return Self::LayingEgg;
        }
        if hunger > 0.0 && options.food_in_reach {
            return Self::SeekingFood;
        }
        if health < 1.0 {
            return Self::Healing;
        }
        if Self::open_to_mate(maturity, desire) && options.mate_in_reach {
            return Self::SeekingPartner;
        }
        let mut rng = rng();

        let just_wait = rng.random_bool(0.5);

        if just_wait {
            return Self::Wait;
        }

        Self::Wandering
    }

    /// True when mating would win once every higher, currently possible need is set aside.
    pub fn would_mate(
        is_pregnant: bool,
        hunger: f64,
        maturity: f64,
        health: f64,
        desire: f64,
        can_lay: bool,
        food_in_reach: bool,
    ) -> bool {
        if maturity >= 1.0 || health <= 0.0 {
            return false;
        }
        if is_pregnant && can_lay {
            return false;
        }
        if hunger > 0.0 && food_in_reach {
            return false;
        }
        if health < 1.0 {
            return false;
        }
        Self::open_to_mate(maturity, desire)
    }

    const fn open_to_mate(maturity: f64, desire: f64) -> bool {
        desire >= 1.0 && maturity > 0.3
    }
}

/// `true` = male, `false` = female. Females receive `Pregnant` on mating.
#[derive(Component, Clone, Copy, Debug)]
pub struct Gender(pub bool);

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

#[derive(Component)]
pub struct Trunk;
