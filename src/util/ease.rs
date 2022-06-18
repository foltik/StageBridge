pub fn dt(t: f32, delta: f32) -> f32 {
    (t + delta).rem_euclid(1.0)
}

pub fn lerp(a: f32, b: f32) -> impl Fn(f32) -> f32 {
    move |fr| a + fr * (b - a)
}

pub fn ilerp(a: f32, b: f32) -> impl Fn(f32) -> f32 {
    move |fr| (fr - a) / (b - a)
}

pub fn map(a0: f32, a1: f32, b0: f32, b1: f32) -> impl Fn(f32) -> f32 {
    move |fr| lerp(b0, b1)(ilerp(a0, a1)(fr))
}

pub fn project(min: f32, max: f32) -> impl Fn(f32) -> f32 {
    move |f| max * (1.0 - min) * f + min
}

pub fn u8(fr: f32) -> u8 {
    (fr * 255.0).floor() as u8
}

pub fn tri(pd: f32) -> impl Fn(f32) -> f32 {
    move |f| (((2.0 * f) % (2.0 * pd)) - pd).abs() / pd
}

pub fn saw_up(pd: f32) -> impl Fn(f32) -> f32 {
    move |f| (f % pd) / pd
}

pub fn saw_down(pd: f32) -> impl Fn(f32) -> f32 {
    move |f| 1.0 - ((f % pd) / pd)
}

pub fn mix(a: f32, b: f32, fr: f32) -> f32 {
    a + (b - a) * fr
}

pub fn step(v: f32, thres: f32) -> f32 {
    if v < thres {
        0.0
    } else {
        1.0
    }
}