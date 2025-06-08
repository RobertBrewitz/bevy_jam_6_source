pub fn quartic_bump(t: f32) -> f32 {
    16. * t * t * (1. - t) * (1. - t)
}

pub fn quartic_bump_f64(t: f64) -> f64 {
    16. * t * t * (1. - t) * (1. - t)
}

/// max between 0. and 1.
pub fn quartic_bump_max(t: f32, max: f32) -> f32 {
    quartic_bump(t) * max
}

/// max between 0. and 1.
pub fn quartic_bump_max_f64(t: f64, max: f64) -> f64 {
    quartic_bump_f64(t) * max
}

/// min between 0. and 1.
pub fn quartic_bump_min(t: f32, min: f32) -> f32 {
    min + (1. - min) * quartic_bump(t)
}

/// min between 0. and 1.
pub fn quartic_bump_min_f64(t: f64, min: f64) -> f64 {
    min + (1. - min) * quartic_bump_f64(t)
}

/// max between 0. and 1.
/// min between 0. and 1.
pub fn quartic_bump_min_max(t: f32, min: f32, max: f32) -> f32 {
    min + (max - min) * quartic_bump(t)
}

/// max between 0. and 1.
/// min between 0. and 1.
pub fn quartic_bump_min_max_f64(t: f64, min: f64, max: f64) -> f64 {
    min + (max - min) * quartic_bump_f64(t)
}

/// k is the narrowness of the bump, the larger the k, the narrower the bump
pub fn bump_logistic(t: f32, k: f32) -> f32 {
    let l = 1. / (1. + (-k * (t - 0.5)).exp());
    4. * l * (1. - l)
}

/// k is the narrowness of the bump, the larger the k, the narrower the bump
pub fn bump_logistic_f64(t: f64, k: f64) -> f64 {
    let l = 1. / (1. + (-k * (t - 0.5)).exp());
    4. * l * (1. - l)
}
