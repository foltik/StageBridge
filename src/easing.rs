use std::f64::consts::PI;

pub fn dt(t: f64, delta: f64) -> f64 {
    (t + delta).rem_euclid(1.0)
}

pub fn lerp(a: f64, b: f64) -> impl Fn(f64) -> f64 {
    move |fr| a + fr * (b - a)
}

pub fn ilerp(a: f64, b: f64) -> impl Fn(f64) -> f64 {
    move |fr| (fr - a) / (b - a)
}

pub fn map(a0: f64, a1: f64, b0: f64, b1: f64) -> impl Fn(f64) -> f64 {
    move |fr| lerp(b0, b1)(ilerp(a0, a1)(fr))
}

pub fn project(min: f64, max: f64) -> impl Fn(f64) -> f64 {
    move |fr| max * (1.0 - min) * fr + min
}

pub fn cover(amt: f64) -> impl Fn(f64) -> f64 {
    move |fr| (fr * amt) + ((1.0 - amt) / 2.0)
}

pub fn mix(a: f64, b: f64, fr: f64) -> f64 {
    a + (b - a) * fr
}

pub fn step(v: f64, thres: f64) -> f64 {
    if v < thres {
        0.0
    } else {
        1.0
    }
}



pub fn sin(pd: f64) -> impl Fn(f64) -> f64 {
    move |t| 0.5 * ((2.0*PI*t)/pd + PI).cos() + 0.5
}

pub fn tri(pd: f64) -> impl Fn(f64) -> f64 {
    move |t| (((2.0 * t) % (2.0 * pd)) - pd).abs() / pd
}

pub fn saw_up(pd: f64) -> impl Fn(f64) -> f64 {
    move |t| (t % pd) / pd
}

pub fn saw_down(pd: f64) -> impl Fn(f64) -> f64 {
    move |t| 1.0 - ((t % pd) / pd)
}

pub fn square(pd: f64, duty: f64, t: f64) -> f64 {
    if (t % pd) <= pd * duty {
        1.0
    } else {
        0.0
    }
}




pub fn u8(fr: f64) -> u8 {
    (fr * 255.0).floor() as u8
}
