use crate::location;
use colored::*;
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
        let inc = |stat: i32, rate: f64| stat + randomized(stat as f64 * rate);

        self.level += 1;
        self.max_hp = inc(self.max_hp, 0.3);
        self.strength = inc(self.strength, 0.1);
        self.speed = inc(self.speed, 0.1);
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

    // FIXME this is all temporary code, need to factor out or to trait
    pub fn display_at(&self, location: &location::Location) -> String {
        format!(
            "{}{}{}@{}",
            self,
            self.hp_display(),
            self.xp_display(),
            location
        )
    }

    fn hp_display(&self) -> String {
        // FIXME this sometimes can still look unfilled at 100%
        let current_units = (self.current_hp as f64 * 4.0 / self.max_hp as f64).ceil() as i32;
        let green = (0..current_units).map(|_| "x").collect::<String>().green();
        let red = (0..(4 - current_units))
            .map(|_| "-")
            .collect::<String>()
            .red();
        format!("[{}{}]", green, red)
    }

    // FIXME duplicated
    fn xp_display(&self) -> String {
        let current_units = self.xp * 4 / self.xp_for_next();
        let green = (0..current_units).map(|_| "x").collect::<String>().cyan();
        let red = (0..(4 - current_units))
            .map(|_| "-")
            .collect::<String>()
            .bright_black();
        format!("[{}{}]", green, red)
    }
}

/// add +/- 10% variance to a f64
/// may make more generic in the future
fn randomized(value: f64) -> i32 {
    let mut rng = rand::thread_rng();
    let min = (value - value * 0.1).floor() as i32;
    let max = (value + value * 0.1).ceil() as i32;
    rng.gen_range(min..=max)
}

impl std::fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = if self.is_dead() {
            "\u{1F480}".to_string()
        } else {
            self.level.to_string()
        };

        // FIXME ugly
        let name = if self.name == "hero" {
            // FIXME use correct padding
            " hero".bold().to_string()
        } else {
            self.name.yellow().bold().to_string()
        };

        write!(f, "{}[{}]", name, level)
    }
}
