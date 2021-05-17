#![allow(dead_code)]

use crate::location;
use rand::Rng;
use std::cmp::max;

pub trait Randomizer {
    fn should_enemy_appear(&self, _distance: &location::Distance) -> bool {
        true
    }

    fn bribe_succeeds(&self) -> bool {
        false
    }

    fn run_away_succeeds(&self, _player_level: i32, _enemy_level: i32) -> bool {
        false
    }

    fn enemy_delta(&self) -> i32 {
        0
    }

    fn damage(&self, value: i32) -> i32 {
        value
    }

    fn should_critical(&self) -> bool {
        false
    }

    fn should_miss(&self, _attacker_speed: i32, _receiver_speed: i32) -> bool {
        false
    }

    fn gold_gained(&self, base: i32) -> i32 {
        base
    }

    fn stat(&self, current: i32, rate: f64) -> i32 {
        current + (current as f64 * rate).ceil() as i32
    }
}

/// This struct exposes functions to deal with any element of the game that
/// needs to incorporate randomness.
/// It basically wraps all calls to the rand crate, allowing to turn it off
/// during testing to make the logic deterministic.
// DISCLAIMER: I'm not convinced this is a good idea.
pub struct DefaultRandomizer {}

impl Randomizer for DefaultRandomizer {
    fn should_enemy_appear(&self, distance: &location::Distance) -> bool {
        let mut rng = rand::thread_rng();

        match distance {
            location::Distance::Near(_) => rng.gen_ratio(1, 3),
            location::Distance::Mid(_) => rng.gen_ratio(1, 2),
            location::Distance::Far(_) => rng.gen_ratio(2, 3),
        }
    }

    fn bribe_succeeds(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_ratio(1, 2)
    }

    fn run_away_succeeds(&self, player_level: i32, enemy_level: i32) -> bool {
        let mut rng = rand::thread_rng();
        match player_level {
            pl if pl == enemy_level => rng.gen_ratio(1, 3),
            pl if pl > enemy_level => rng.gen_ratio(2, 3),
            _ => rng.gen_ratio(1, 5),
        }
    }

    fn enemy_delta(&self) -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(-1..2)
    }

    /// add +/- 20% variance to a the damage
    fn damage(&self, value: i32) -> i32 {
        let value = value as f64;

        let mut rng = rand::thread_rng();
        let min = (value * 0.8).floor() as i32;
        let max = (value * 1.2).ceil() as i32;
        rng.gen_range(min..=max)
    }

    fn should_critical(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_ratio(1, 20)
    }

    fn should_miss(&self, attacker_speed: i32, receiver_speed: i32) -> bool {
        if receiver_speed > attacker_speed {
            let ratio = receiver_speed / attacker_speed;
            let ratio = max(1, 5 - ratio) as u32;
            let mut rng = rand::thread_rng();
            return rng.gen_ratio(1, ratio);
        }
        false
    }

    fn gold_gained(&self, base: i32) -> i32 {
        let mut rng = rand::thread_rng();
        let min = (base as f64 * 0.6) as i32;
        let max = (base as f64 * 1.3) as i32;
        rng.gen_range(min..=max)
    }

    fn stat(&self, current: i32, rate: f64) -> i32 {
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

impl Randomizer for TestRandomizer{}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increase_stat() {
        // we explicitly test the default implementation, not test one here
        let rand = DefaultRandomizer{};

        // current hp lvl1: increase in .3 +/- .15
        let value = rand.stat(20, 0.3);
        assert!((23..=29).contains(&value), "value was {}", value);

        // current strength lvl1
        let value = rand.stat(10, 0.1);
        assert!((11..=12).contains(&value), "value was {}", value);

        // current speed lvl1
        let value = rand.stat(5, 0.1);
        assert_eq!(6, value);

        // ~ hp lvl2
        let value = rand.stat(26, 0.3);
        assert!((30..=38).contains(&value), "value was {}", value);

        // ~ hp lvl3
        let value = rand.stat(34, 0.3);
        assert!((39..=49).contains(&value), "value was {}", value);

        // small numbers
        let value = rand.stat(3, 0.07);
        assert_eq!(4, value);
    }
}
