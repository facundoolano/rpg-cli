use rand::Rng;
use std::cmp::max;

pub struct Randomizer {}

#[cfg(not(test))]
impl Randomizer {
    pub fn enemy_delta() -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(-1..2)
    }

    pub fn should_enemy_appear() -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_ratio(1, 3)
    }

    pub fn stat(current: i32, rate: f64) -> i32 {
        // if rate is .3, increase can be in .15-.45
        let current_f = current as f64;
        let min_value = max(1, (current_f * (rate - rate / 2.0)).round() as i32);
        let max_value = max(1, (current_f * rate + rate / 2.0).round() as i32);

        let mut rng = rand::thread_rng();
        current + rng.gen_range(min_value..=max_value)
    }

    /// add +/- 20% variance to a the damage
    pub fn damage(value: i32) -> i32 {
        let value = value as f64;

        let mut rng = rand::thread_rng();
        let min = (value - value * 0.2).floor() as i32;
        let max = (value + value * 0.2).ceil() as i32;
        rng.gen_range(min..=max)
    }
}

#[cfg(test)]
impl Randomizer {
    pub fn enemy_delta() -> i32 {
        0
    }

    pub fn should_enemy_appear() -> bool {
        true
    }

    pub fn stat(current: i32, rate: f64) -> i32 {
        current + (current as f64 * rate).ceil() as i32
    }

    pub fn damage(value: i32) -> i32 {
        value
    }
}
