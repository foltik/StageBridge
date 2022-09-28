use std::f32::consts::PI;

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
    move |fr| max * (1.0 - min) * fr + min
}

pub fn cover(amt: f32) -> impl Fn(f32) -> f32 {
    move |fr| (fr * amt) + ((1.0 - amt) / 2.0)
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



pub fn sin(pd: f32) -> impl Fn(f32) -> f32 {
    move |t| 0.5 * ((2.0*PI*t)/pd + PI).cos() + 0.5
}

pub fn tri(pd: f32) -> impl Fn(f32) -> f32 {
    move |t| (((2.0 * t) % (2.0 * pd)) - pd).abs() / pd
}

pub fn saw_up(pd: f32) -> impl Fn(f32) -> f32 {
    move |t| (t % pd) / pd
}

pub fn saw_down(pd: f32) -> impl Fn(f32) -> f32 {
    move |t| 1.0 - ((t % pd) / pd)
}

pub fn square(pd: f32, duty: f32, t: f32) -> f32 {
    if (t % pd) <= pd * duty {
        1.0
    } else {
        0.0
    }
}




pub fn u8(fr: f32) -> u8 {
    (fr * 255.0).floor() as u8
}
