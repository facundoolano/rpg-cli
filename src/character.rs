use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::max;

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
            max_hp: 20,
            current_hp: 20,
            strength: 10,
            speed: 5,
        };

        for _ in 1..level {
            character.increase_level();
        }

        character
    }

    /// Raise the level and all the character stats.
    fn increase_level(&mut self) {
        // TODO different rates by char class and enemy type -> parametrized enum. could include start values as well
        let inc =
            |stat: i32, rate: f64| (stat as f64 + randomized(stat as f64 * rate)).round() as i32;

        self.level += 1;
        self.max_hp = inc(self.max_hp, 0.3);
        self.strength = inc(self.strength, 0.1);
        self.speed = inc(self.speed, 0.1);
    }

    /// Add to the accumulated experience points, possibly increasing the level.
    fn add_experience(&mut self, xp: i32) {
        self.xp += xp;
        let for_next = self.xp_for_next();
        if self.xp >= for_next {
            self.increase_level();
            self.xp -= for_next;
        }
    }

    pub fn receive_damage(&mut self, damage: i32) {
        self.current_hp = max(0, self.current_hp - damage);
    }

    pub fn is_dead(&self) -> bool {
        self.current_hp == 0
    }

    pub fn heal(&mut self) {
        self.current_hp = self.max_hp;
    }

    /// How many experience points are required to move to the next level.
    fn xp_for_next(&self) -> i32 {
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

/// add +/- 10% variance to a f64
/// may make more generic in the future
fn randomized(value: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let min = value * 0.9;
    let max = value * 1.1;
    rng.gen_range(min..=max)
}
