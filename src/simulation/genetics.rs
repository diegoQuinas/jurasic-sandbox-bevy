use super::components::DinosaurStats;

pub fn blend_dino_stats(father: &DinosaurStats, mother: &DinosaurStats) -> DinosaurStats {
    DinosaurStats {
        reproduction_desire: blend_average_f64(
            father.reproduction_desire,
            mother.reproduction_desire,
        ),
        color: blend_colors(father.color, mother.color),
        starvation_resistance: blend_average_f64(
            father.starvation_resistance,
            mother.starvation_resistance,
        ),
        generation: father.generation.saturating_add(1),
    }
}

fn blend_colors(father: (u8, u8, u8), mother: (u8, u8, u8)) -> (u8, u8, u8) {
    (
        ((mother.0 as u16 + father.0 as u16) / 2) as u8,
        ((mother.1 as u16 + father.1 as u16) / 2) as u8,
        ((mother.2 as u16 + father.2 as u16) / 2) as u8,
    )
}

fn blend_average_f64(father: f64, mother: f64) -> f64 {
    (father + mother) / 2.0
}
