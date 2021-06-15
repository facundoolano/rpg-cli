#![allow(dead_code)]

use crate::character::StatusEffect;
use crate::game::battle::AttackType;
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

    fn run_away_succeeds(&self, player_level: i32, enemy_level: i32) -> bool;

    fn enemy_level(&self, level: i32) -> i32;

    fn damage(&self, value: i32) -> i32;

    fn attack_type(
        &self,
        produced_status: Option<StatusEffect>,
        attacker_speed: i32,
        receiver_speed: i32,
    ) -> AttackType;

    fn gold_gained(&self, base: i32) -> i32;

    fn stat_increase(&self, increase: i32) -> i32;

    fn range(&self, max: i32) -> i32;
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

    fn run_away_succeeds(&self, player_level: i32, enemy_level: i32) -> bool {
        let mut rng = rand::thread_rng();
        match player_level {
            pl if pl == enemy_level => rng.gen_ratio(1, 3),
            pl if pl > enemy_level => rng.gen_ratio(2, 3),
            _ => rng.gen_ratio(1, 5),
        }
    }

    fn enemy_level(&self, level: i32) -> i32 {
        let mut rng = rand::thread_rng();
        max(1, level + rng.gen_range(-1..2))
    }

    /// add +/- 20% variance to a the damage
    fn damage(&self, value: i32) -> i32 {
        let value = value as f64;

        let mut rng = rand::thread_rng();
        let min_val = (value * 0.8).floor() as i32;
        let max_val = (value * 1.2).ceil() as i32;
        max(1, rng.gen_range(min_val..=max_val))
    }

    fn attack_type(
        &self,
        produced_status: Option<StatusEffect>,
        attacker_speed: i32,
        receiver_speed: i32,
    ) -> AttackType {
        // TODO this could maybe flattened to a single random call
        // instead of trial and error
        if is_miss(attacker_speed, receiver_speed) {
            AttackType::Miss
        } else if is_critical() {
            AttackType::Critical
        } else if let Some(status) = status_effect(produced_status) {
            AttackType::Effect(status)
        } else {
            AttackType::Regular
        }
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
}

fn is_critical() -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_ratio(1, 20)
}

fn is_miss(attacker_speed: i32, receiver_speed: i32) -> bool {
    if receiver_speed > attacker_speed {
        let ratio = receiver_speed / attacker_speed;
        let ratio = max(1, 5 - ratio) as u32;
        let mut rng = rand::thread_rng();
        return rng.gen_ratio(1, ratio);
    }
    false
}

fn status_effect(produced_status: Option<StatusEffect>) -> Option<StatusEffect> {
    let mut rng = rand::thread_rng();
    if rng.gen_ratio(1, 20){
        produced_status
    } else {
        None
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

    fn run_away_succeeds(&self, _player_level: i32, _enemy_level: i32) -> bool {
        false
    }

    fn enemy_level(&self, _level: i32) -> i32 {
        0
    }

    fn damage(&self, value: i32) -> i32 {
        value
    }

    fn attack_type(
        &self,
        _produced_status: Option<StatusEffect>,
        _attacker_speed: i32,
        _receiver_speed: i32,
    ) -> AttackType {
        AttackType::Regular
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
