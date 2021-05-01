use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::max;

// TODO these values could be different for different character classes
// and pick them based on some enum.
// We could start off with Hero and Enemy enums for now
const START_HP: i32 = 20;
const START_STRENGTH: i32 = 10;
const START_SPEED: i32 = 10;
const HP_RATE: f64 = 0.3;
const STRENGTH_RATE: f64 = 0.1;
const SPEED_RATE: f64 = 0.1;

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    pub name: String,

    pub level: i32,
    pub xp: i32,

    pub max_hp: i32,
    pub current_hp: i32,

    pub strength: i32,
    pub speed: i32,
}

impl Character {
    // we could have a Character trait and separate Player and Enemy structs
    // but there's barely any logic to justify that yet
    pub fn player() -> Self {
        Self::new("hero", 1)
    }

    pub fn new(name: &str, level: i32) -> Self {
        let mut character = Self {
            name: String::from(name),
            level,
            xp: 0,
            max_hp: START_HP,
            current_hp: START_HP,
            strength: START_STRENGTH,
            speed: START_SPEED,
        };

        for _ in 1..level {
            character.increase_level();
        }

        character
    }

    /// Raise the level and all the character stats.
    fn increase_level(&mut self) {
        self.level += 1;
        self.strength = inc(self.strength, STRENGTH_RATE);
        self.speed = inc(self.speed, SPEED_RATE);

        // the current should increase proportionally but not
        // erase previous damage
        let previous_damage = self.max_hp - self.current_hp;
        self.max_hp = inc(self.max_hp, HP_RATE);
        self.current_hp = self.max_hp - previous_damage;
    }

    /// Add to the accumulated experience points, possibly increasing the level.
    pub fn add_experience(&mut self, xp: i32) -> bool {
        self.xp += xp;
        let for_next = self.xp_for_next();
        if self.xp >= for_next {
            self.increase_level();
            self.xp -= for_next;
            return true;
        }
        false
    }

    pub fn receive_damage(&mut self, damage: i32) {
        self.current_hp = max(0, self.current_hp - damage);
    }

    pub fn is_dead(&self) -> bool {
        self.current_hp == 0
    }

    pub fn heal(&mut self) -> i32 {
        let recovered = self.max_hp - self.current_hp;
        self.current_hp = self.max_hp;
        recovered
    }

    /// How many experience points are required to move to the next level.
    pub fn xp_for_next(&self) -> i32 {
        let exp = 1.5;
        let base_xp = 30;
        base_xp * (self.level as f64).powf(exp) as i32
    }

    /// Generate a randomized damage numer based on the attacker strength
    /// and the receiver strength.
    pub fn damage(&self, receiver: &Self) -> i32 {
        // Possible improvements: use different attack and defense stats,
        // incorporate weapon and armor effect.

        let str_10 = self.strength as f64 * 0.1;
        let damage = self.strength as f64 + (self.level - receiver.level) as f64 * str_10;
        max(str_10.ceil() as i32, randomized(damage) as i32)
    }

    /// How many experience points are gained by inflicting damage to an enemy.
    pub fn xp_gained(&self, receiver: &Self, damage: i32) -> i32 {
        // TODO should the player also gain experience by damage received?
        damage * max(1, 1 + receiver.level - self.level)
    }
}

/// Increase a stat by the given rate, with some randomization.
fn inc(current: i32, rate: f64) -> i32 {
    // if rate is .3, increase can be in .15-.45
    let current_f = current as f64;
    let min = std::cmp::max(1, (current_f * (rate - rate / 2.0)).round() as i32);
    let max = (current_f * rate + rate / 2.0).round() as i32;

    let mut rng = rand::thread_rng();
    current + rng.gen_range(min..=max)
}

// FIXME review: this is dubiously implemented and now used in a single place
/// add +/- 10% variance to a f64
/// may make more generic in the future
fn randomized(value: f64) -> i32 {
    let mut rng = rand::thread_rng();
    let min = (value - value * 0.1).floor() as i32;
    let max = (value + value * 0.1).ceil() as i32;
    rng.gen_range(min..=max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let hero = Character::player();

        assert_eq!(1, hero.level);
        assert_eq!(0, hero.xp);
        assert_eq!(START_HP, hero.current_hp);
        assert_eq!(START_HP, hero.max_hp);
        assert_eq!(START_STRENGTH, hero.strength);
        assert_eq!(START_SPEED, hero.speed);
    }

    #[test]
    fn test_increase_stat() {
        // current hp lvl1: increase in .3 +/- .15
        let value = inc(20, 0.3);
        assert!((23..=29).contains(&value), "value was {}", value);

        // current strength lvl1
        let value = inc(10, 0.1);
        assert!((11..=12).contains(&value), "value was {}", value);

        // current speed lvl1
        let value = inc(5, 0.1);
        assert_eq!(6, value);

        // ~ hp lvl2
        let value = inc(26, 0.3);
        assert!((30..=38).contains(&value), "value was {}", value);

        // ~ hp lvl3
        let value = inc(34, 0.3);
        assert!((39..=49).contains(&value), "value was {}", value);
    }

    #[test]
    fn test_increase_level() {
        let mut hero = Character::player();

        // Using hardcoded start/rates so we can assert with specific values
        // TODO add specific test character class that we can assume won't change
        assert_eq!(0.3, HP_RATE);
        assert_eq!(0.1, STRENGTH_RATE);
        assert_eq!(0.1, SPEED_RATE);
        hero.max_hp = 20;
        hero.current_hp = 20;
        hero.strength = 10;
        hero.speed = 5;

        hero.increase_level();
        assert_eq!(2, hero.level);
        assert!((23..=29).contains(&hero.max_hp));
        assert!((11..=12).contains(&hero.strength));
        assert_eq!(6, hero.speed);

        let damage = 7;
        hero.current_hp -= damage;

        hero.increase_level();
        assert_eq!(3, hero.level);
        assert_eq!(hero.current_hp, hero.max_hp - damage);
    }
}
