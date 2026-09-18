use super::components::DinosaurStats;

pub const fn blend_dino_stats(father: DinosaurStats, mother: DinosaurStats) -> DinosaurStats {
    let color = blend_colors(father.color, mother.color);
    let starvation_resistance =
        blend_average_f64(father.starvation_resistance, mother.starvation_resistance);
    let metabolism = blend_average_f64(father.metabolism, mother.metabolism);
    DinosaurStats {
        color,
        starvation_resistance,
        generation: father.generation.saturating_add(1),
        metabolism,
    }
}

const fn blend_colors(father: (u8, u8, u8), mother: (u8, u8, u8)) -> (u8, u8, u8) {
    (
        u16::midpoint(mother.0 as u16, father.0 as u16) as u8,
        u16::midpoint(mother.1 as u16, father.1 as u16) as u8,
        u16::midpoint(mother.2 as u16, father.2 as u16) as u8,
    )
}

const fn blend_average_f64(father: f64, mother: f64) -> f64 {
    f64::midpoint(father, mother)
}
