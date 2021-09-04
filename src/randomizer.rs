#![allow(dead_code)]

use crate::character::StatusEffect;
use crate::location;
use rand::Rng;
use std::cmp::max;

/// This trait exposes functions to deal with any element of the game that
/// needs to incorporate randomness.
/// It basically wraps all calls to the rand crate, allowing to replace it with a
/// noop implementation in tests to make the logic deterministic.
pub trait Randomizer {
    fn should_enemy_appear(&self, distance: &location::Distance) -> bool;

    fn bribe_succeeds(&self) -> bool;

    fn run_away_succeeds(
        &self,
        player_level: i32,
        enemy_level: i32,
        player_speed: i32,
        enemy_speed: i32,
    ) -> bool;

    fn enemy_level(&self, level: i32) -> i32;

    fn damage(&self, value: i32) -> i32;

    fn is_miss(&self, attacker_speed: i32, receiver_speed: i32) -> bool;

    fn is_critical(&self) -> bool;

    fn counter_attack(&self) -> bool;

    fn inflicted(&self, status: Option<(StatusEffect, u32)>) -> Option<StatusEffect>;

    fn gold_gained(&self, base: i32) -> i32;

    fn stat_increase(&self, increase: i32) -> i32;

    fn range(&self, max: i32) -> i32;

    fn gold_chest(&self, distance: &location::Distance) -> bool;
    fn equipment_chest(&self, distance: &location::Distance) -> bool;
    fn ring_chest(&self, distance: &location::Distance) -> bool;
    fn item_chest(&self, distance: &location::Distance) -> bool;
}

#[cfg(not(test))]
/// Get the randomizer instance. This function provides indirection
/// so randomness can be turned off during tests to make them deterministic
pub fn random() -> DefaultRandomizer {
    DefaultRandomizer {}
}

#[cfg(test)]
pub fn random() -> TestRandomizer {
    TestRandomizer {}
}

pub struct DefaultRandomizer;

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

    fn run_away_succeeds(
        &self,
        player_level: i32,
        enemy_level: i32,
        player_speed: i32,
        enemy_speed: i32,
    ) -> bool {
        let level_contrib = if player_level > enemy_level { 1 } else { 0 };

        let speed_contrib = if player_speed > enemy_speed { 2 } else { 0 };

        let mut rng = rand::thread_rng();
        rng.gen_ratio(1 + level_contrib + speed_contrib, 5)
    }

    fn enemy_level(&self, level: i32) -> i32 {
        let mut rng = rand::thread_rng();
        max(1, level + rng.gen_range(-4..5))
    }

    /// add +/- 20% variance to a the damage
    fn damage(&self, value: i32) -> i32 {
        let value = value as f64;

        let mut rng = rand::thread_rng();
        let min_val = (value * 0.8).floor() as i32;
        let max_val = (value * 1.2).ceil() as i32;
        max(1, rng.gen_range(min_val..=max_val))
    }

    fn is_miss(&self, attacker_speed: i32, receiver_speed: i32) -> bool {
        if receiver_speed > attacker_speed {
            let ratio = receiver_speed / attacker_speed;
            let ratio = max(1, 5 - ratio) as u32;
            let mut rng = rand::thread_rng();
            return rng.gen_ratio(1, ratio);
        }
        false
    }

    fn is_critical(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_ratio(1, 20)
    }

    fn counter_attack(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_ratio(1, 2)
    }

    fn inflicted(&self, status: Option<(StatusEffect, u32)>) -> Option<StatusEffect> {
        if let Some((status, ratio)) = status {
            let mut rng = rand::thread_rng();
            if rng.gen_ratio(1, ratio) {
                return Some(status);
            }
        }
        None
    }

    fn gold_gained(&self, base: i32) -> i32 {
        let mut rng = rand::thread_rng();
        let min = (base as f64 * 0.6) as i32;
        let max = (base as f64 * 1.3) as i32;
        rng.gen_range(min..=max)
    }

    fn stat_increase(&self, increase: i32) -> i32 {
        let min_value = max(1, increase / 2);
        let max_value = 3 * increase / 2;

        let mut rng = rand::thread_rng();
        rng.gen_range(min_value..=max_value)
    }

    fn range(&self, max: i32) -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..max)
    }

    fn gold_chest(&self, distance: &location::Distance) -> bool {
        let mut rng = rand::thread_rng();

        match distance {
            location::Distance::Near(_) => rng.gen_ratio(6, 30),
            location::Distance::Mid(_) => rng.gen_ratio(7, 30),
            location::Distance::Far(_) => rng.gen_ratio(4, 30),
        }
    }

    fn equipment_chest(&self, distance: &location::Distance) -> bool {
        let mut rng = rand::thread_rng();

        match distance {
            location::Distance::Near(_) => false,
            location::Distance::Mid(_) => rng.gen_ratio(2, 30),
            location::Distance::Far(_) => rng.gen_ratio(4, 30),
        }
    }

    fn ring_chest(&self, distance: &location::Distance) -> bool {
        let mut rng = rand::thread_rng();

        match distance {
            location::Distance::Near(_) => false,
            location::Distance::Mid(_) => rng.gen_ratio(3, 30),
            location::Distance::Far(_) => rng.gen_ratio(5, 30),
        }
    }

    fn item_chest(&self, distance: &location::Distance) -> bool {
        let mut rng = rand::thread_rng();

        match distance {
            location::Distance::Near(_) => rng.gen_ratio(1, 50),
            location::Distance::Mid(_) => rng.gen_ratio(5, 50),
            location::Distance::Far(_) => rng.gen_ratio(10, 50),
        }
    }
}

/// The test randomizer just exposes the same functions as the default one
/// but return deterministic results.
pub struct TestRandomizer;

impl Randomizer for TestRandomizer {
    fn should_enemy_appear(&self, _distance: &location::Distance) -> bool {
        true
    }

    fn bribe_succeeds(&self) -> bool {
        false
    }

    fn run_away_succeeds(
        &self,
        _player_level: i32,
        _enemy_level: i32,
        _player_speed: i32,
        _enemy_speed: i32,
    ) -> bool {
        false
    }

    fn enemy_level(&self, level: i32) -> i32 {
        level
    }

    fn damage(&self, value: i32) -> i32 {
        value
    }

    fn is_miss(&self, _attacker_speed: i32, _receiver_speed: i32) -> bool {
        false
    }

    fn is_critical(&self) -> bool {
        false
    }

    fn counter_attack(&self) -> bool {
        true
    }

    fn inflicted(&self, _status: Option<(StatusEffect, u32)>) -> Option<StatusEffect> {
        None
    }

    fn gold_gained(&self, base: i32) -> i32 {
        base
    }

    fn stat_increase(&self, increase: i32) -> i32 {
        increase
    }

    fn range(&self, max: i32) -> i32 {
        max
    }

    fn gold_chest(&self, _distance: &location::Distance) -> bool {
        false
    }

    fn equipment_chest(&self, _distance: &location::Distance) -> bool {
        false
    }

    fn item_chest(&self, _distance: &location::Distance) -> bool {
        false
    }

    fn ring_chest(&self, _distance: &location::Distance) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increase_stat() {
        let rand = DefaultRandomizer {};

        // current hp lvl1
        let value = rand.stat_increase(7);
        assert!((3..=10).contains(&value), "value was {}", value);

        // current strength lvl1
        let value = rand.stat_increase(3);
        assert!((1..=4).contains(&value), "value was {}", value);

        // current speed lvl1
        let value = rand.stat_increase(2);
        assert!((1..=3).contains(&value), "value was {}", value);

        // small increase
        let value = rand.stat_increase(1);
        assert!((1..=2).contains(&value), "value was {}", value);
    }
}
