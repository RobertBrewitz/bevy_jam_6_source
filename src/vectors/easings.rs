const C4: f32 = std::f32::consts::TAU / 3.0;
const C4_F64: f64 = std::f64::consts::TAU / 3.0;
const C5: f32 = std::f32::consts::TAU / 4.5;
const C5_F64: f64 = std::f64::consts::TAU / 4.5;

/// https://easings.net/#easeInElastic
pub fn ease_in_elastic(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if 1.0 <= t {
        1.0
    } else {
        -(2f32.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * C4).sin()
    }
}

/// https://easings.net/#easeInElastic
pub fn ease_in_elastic_f64(t: f64) -> f64 {
    if t <= 0.0 {
        0.0
    } else if 1.0 <= t {
        1.0
    } else {
        -(2f64.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * C4_F64).sin()
    }
}

/// https://easings.net/#easeOutElastic
pub fn ease_out_elastic(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if 1.0 <= t {
        1.0
    } else {
        2f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0
    }
}

/// https://easings.net/#easeOutElastic
pub fn ease_out_elastic_f64(t: f64) -> f64 {
    if t <= 0.0 {
        0.0
    } else if 1.0 <= t {
        1.0
    } else {
        2f64.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4_F64).sin() + 1.0
    }
}

/// https://easings.net/#easeInOutElastic
pub fn ease_in_out_elastic(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if 1.0 <= t {
        1.0
    } else if t < 0.5 {
        -(2f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0
    } else {
        (2f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0
            + 1.0
    }
}

/// https://easings.net/#easeInOutElastic
pub fn ease_in_out_elastic_f64(t: f64) -> f64 {
    if t <= 0.0 {
        0.0
    } else if 1.0 <= t {
        1.0
    } else if t < 0.5 {
        -(2f64.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C5_F64).sin())
            / 2.0
    } else {
        (2f64.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C5_F64).sin())
            / 2.0
            + 1.0
    }
}

/// https://easings.net/#easeInBounce
pub fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

/// https://easings.net/#easeInBounce
pub fn ease_in_bounce_f64(t: f64) -> f64 {
    1.0 - ease_out_bounce_f64(1.0 - t)
}

/// https://easings.net/#easeOutBounce
pub fn ease_out_bounce(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        N1 * (t - 1.5 / D1).powi(2) + 0.75
    } else if t < 2.5 / D1 {
        N1 * (t - 2.25 / D1).powi(2) + 0.9375
    } else {
        N1 * (t - 2.625 / D1).powi(2) + 0.984375
    }
}

/// https://easings.net/#easeOutBounce
pub fn ease_out_bounce_f64(t: f64) -> f64 {
    const N1: f64 = 7.5625;
    const D1: f64 = 2.75;
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        N1 * (t - 1.5 / D1).powi(2) + 0.75
    } else if t < 2.5 / D1 {
        N1 * (t - 2.25 / D1).powi(2) + 0.9375
    } else {
        N1 * (t - 2.625 / D1).powi(2) + 0.984375
    }
}

/// https://easings.net/#easeInOutBounce
pub fn ease_in_out_bounce(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

/// https://easings.net/#easeInOutBounce
pub fn ease_in_out_bounce_f64(t: f64) -> f64 {
    if t < 0.5 {
        (1.0 - ease_out_bounce_f64(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce_f64(2.0 * t - 1.0)) / 2.0
    }
}
