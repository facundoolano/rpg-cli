#![allow(dead_code)]

use crate::location;
use rand::Rng;
use std::cmp::max;

/// This struct exposes functions to deal with any element of the game that
/// needs to incorporate randomness.
/// It basically wraps all calls to the rand crate, allowing to turn it off
/// during testing to make the logic deterministic.
// DISCLAIMER: I'm not convinced this is a good idea.
pub struct DefaultRandomizer {}

impl DefaultRandomizer {
    pub fn should_enemy_appear(distance: &location::Distance) -> bool {
        let mut rng = rand::thread_rng();

        match distance {
            location::Distance::Near(_) => rng.gen_ratio(1, 3),
            location::Distance::Mid(_) => rng.gen_ratio(1, 2),
            location::Distance::Far(_) => rng.gen_ratio(2, 3),
        }
    }

    pub fn enemy_delta() -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(-1..2)
    }

    /// add +/- 20% variance to a the damage
    pub fn damage(value: i32) -> i32 {
        let value = value as f64;

        let mut rng = rand::thread_rng();
        let min = (value * 0.8).floor() as i32;
        let max = (value * 1.2).ceil() as i32;
        rng.gen_range(min..=max)
    }

    pub fn should_critical() -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_ratio(1, 20)
    }

    pub fn should_miss(attacker_speed: i32, receiver_speed: i32) -> bool {
        if receiver_speed > attacker_speed {
            let ratio = receiver_speed / attacker_speed;
            let ratio = max(1, 5 - ratio) as u32;
            let mut rng = rand::thread_rng();
            return rng.gen_ratio(1, ratio);
        }
        false
    }

    pub fn gold_gained(base: i32) -> i32 {
        let mut rng = rand::thread_rng();
        let min = (base as f64 * 0.6) as i32;
        let max = (base as f64 * 1.3) as i32;
        rng.gen_range(min..=max)
    }

    pub fn stat(current: i32, rate: f64) -> i32 {
        // if rate is .3, increase can be in .15-.45
        let current_f = current as f64;
        let min_value = max(1, (current_f * (rate - rate / 2.0)).round() as i32);
        let max_value = max(1, (current_f * rate + rate / 2.0).round() as i32);

        let mut rng = rand::thread_rng();
        current + rng.gen_range(min_value..=max_value)
    }
}

/// The test randomizer just exposes the same functions as the default one
/// but return deterministic results.
pub struct TestRandomizer {}

impl TestRandomizer {
    pub fn should_enemy_appear(_distance: &location::Distance) -> bool {
        true
    }

    pub fn enemy_delta() -> i32 {
        0
    }

    pub fn damage(value: i32) -> i32 {
        value
    }

    pub fn should_critical() -> bool {
        false
    }

    pub fn should_miss(_attacker_speed: i32, _receiver_speed: i32) -> bool {
        false
    }

    pub fn gold_gained(base: i32) -> i32 {
        base
    }

    pub fn stat(current: i32, rate: f64) -> i32 {
        current + (current as f64 * rate).ceil() as i32
    }
}

/// The randomizer is exposed through a type alias so it can be "turned off"
/// in tests.
#[cfg(not(test))]
pub type Randomizer = DefaultRandomizer;
#[cfg(test)]
pub type Randomizer = TestRandomizer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increase_stat() {
        // we explicitly test the default implementation, not test one here

        // current hp lvl1: increase in .3 +/- .15
        let value = DefaultRandomizer::stat(20, 0.3);
        assert!((23..=29).contains(&value), "value was {}", value);

        // current strength lvl1
        let value = DefaultRandomizer::stat(10, 0.1);
        assert!((11..=12).contains(&value), "value was {}", value);

        // current speed lvl1
        let value = DefaultRandomizer::stat(5, 0.1);
        assert_eq!(6, value);

        // ~ hp lvl2
        let value = DefaultRandomizer::stat(26, 0.3);
        assert!((30..=38).contains(&value), "value was {}", value);

        // ~ hp lvl3
        let value = DefaultRandomizer::stat(34, 0.3);
        assert!((39..=49).contains(&value), "value was {}", value);

        // small numbers
        let value = DefaultRandomizer::stat(3, 0.07);
        assert_eq!(4, value);
    }
}
