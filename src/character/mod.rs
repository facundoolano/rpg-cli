use crate::item::Equipment;
use serde::{Deserialize, Serialize};
use std::cmp::max;

pub mod class;
use crate::item;
use crate::randomizer::Randomizer;
use class::Class;

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    #[serde(skip, default = "default_class")]
    class: &'static Class,
    pub sword: Option<item::Sword>,
    pub shield: Option<item::Shield>,

    pub level: i32,
    pub xp: i32,

    pub max_hp: i32,
    pub current_hp: i32,

    pub strength: i32,
    pub speed: i32,
}

// Always attach the static hero class to deserialized characters
fn default_class() -> &'static Class {
    &Class::HERO
}

impl Character {
    pub fn player() -> Self {
        Self::new(&Class::HERO, 1)
    }

    pub fn enemy(level: i32, distance_from_home: i32) -> Self {
        Self::new(Class::random_enemy(distance_from_home), level)
    }

    pub fn name(&self) -> String {
        self.class.name.to_string()
    }

    pub fn is_player(&self) -> bool {
        // kind of ugly but does the job
        self.class.name == "hero"
    }

    fn new(class: &'static Class, level: i32) -> Self {
        let mut character = Self {
            class,
            sword: None,
            shield: None,
            level: 1,
            xp: 0,
            max_hp: class.start_hp,
            current_hp: class.start_hp,
            strength: class.start_strength,
            speed: class.start_speed,
        };

        for _ in 1..level {
            character.increase_level();
        }

        character
    }

    /// Raise the level and all the character stats.
    fn increase_level(&mut self) {
        self.level += 1;
        self.strength = Randomizer::stat(self.strength, self.class.strength_rate);
        self.speed = Randomizer::stat(self.speed, self.class.speed_rate);

        // the current should increase proportionally but not
        // erase previous damage
        let previous_damage = self.max_hp - self.current_hp;
        self.max_hp = Randomizer::stat(self.max_hp, self.class.hp_rate);
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
        let base_xp = 30.0;
        (base_xp * (self.level as f64).powf(exp)) as i32
    }

    /// Generate a randomized damage numer based on the attacker strength
    /// and the receiver strength.
    pub fn damage(&self, receiver: &Self) -> i32 {
        let damage = self.attack() - receiver.deffense();
        max(1, Randomizer::damage(damage))
    }

    fn attack(&self) -> i32 {
        let sword_str = self.sword.as_ref().map_or(0, |s| s.strength());
        self.strength + sword_str
    }

    fn deffense(&self) -> i32 {
        // we could incorporate strength here, but it's not clear if wouldn't just be noise
        // and it could also made it hard to make damage to stronger enemies
        self.shield.as_ref().map_or(0, |s| s.strength())
    }

    /// How many experience points are gained by inflicting damage to an enemy.
    pub fn xp_gained(&self, receiver: &Self, damage: i32) -> i32 {
        // should the player also gain experience by damage received?

        if receiver.level > self.level {
            damage * (1 + receiver.level - self.level)
        } else {
            damage / (1 + self.level - receiver.level)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CLASS: Class = Class {
        name: "test",
        start_hp: 25,
        start_strength: 10,
        start_speed: 5,

        hp_rate: 0.3,
        strength_rate: 0.1,
        speed_rate: 0.1,
    };

    fn new_char() -> Character {
        Character::new(&TEST_CLASS, 1)
    }

    #[test]
    fn test_new() {
        let hero = new_char();

        assert_eq!(1, hero.level);
        assert_eq!(0, hero.xp);

        assert_eq!(TEST_CLASS.start_hp, hero.current_hp);
        assert_eq!(TEST_CLASS.start_hp, hero.max_hp);
        assert_eq!(TEST_CLASS.start_strength, hero.strength);
        assert_eq!(TEST_CLASS.start_speed, hero.speed);
    }

    #[test]
    fn test_increase_level() {
        let mut hero = new_char();

        // assert what we're assuming are the params in the rest of the test
        assert_eq!(0.3, TEST_CLASS.hp_rate);
        assert_eq!(0.1, TEST_CLASS.strength_rate);
        assert_eq!(0.1, TEST_CLASS.speed_rate);

        hero.max_hp = 20;
        hero.current_hp = 20;
        hero.strength = 10;
        hero.speed = 5;

        hero.increase_level();
        assert_eq!(2, hero.level);
        assert_eq!(26, hero.max_hp);
        assert_eq!(11, hero.strength);
        assert_eq!(6, hero.speed);

        let damage = 7;
        hero.current_hp -= damage;

        hero.increase_level();
        assert_eq!(3, hero.level);
        assert_eq!(hero.current_hp, hero.max_hp - damage);
    }

    #[test]
    fn test_damage() {
        let mut hero = new_char();
        let mut foe = new_char();

        // 1 vs 1
        hero.strength = 10;
        foe.strength = 10;
        assert_eq!(10, hero.damage(&foe));

        // level 1 vs level 2
        foe.level = 2;
        foe.strength = 15;
        assert_eq!(10, hero.damage(&foe));

        // level 2 vs level 1
        assert_eq!(15, foe.damage(&hero));

        // level 1 vs level 5
        foe.level = 5;
        foe.strength = 40;
        assert_eq!(10, hero.damage(&foe));

        // level 5 vs level 1
        assert_eq!(40, foe.damage(&hero));
    }

    #[test]
    fn test_xp_gained() {
        let hero = new_char();
        let mut foe = new_char();
        let damage = 10;

        // 1 vs 1 -- no level-based effect
        let xp = hero.xp_gained(&foe, damage);
        assert_eq!(damage, xp);

        // level 1 vs level 2
        foe.level = 2;
        let xp = hero.xp_gained(&foe, damage);
        assert_eq!(2 * damage, xp);

        // level 2 vs level 1
        let xp = foe.xp_gained(&hero, damage);
        assert_eq!(damage / 2, xp);

        // level 1 vs level 5
        foe.level = 5;
        let xp = hero.xp_gained(&foe, damage);
        assert_eq!(5 * damage, xp);

        // level 5 vs level 1
        let xp = foe.xp_gained(&hero, damage);
        assert_eq!(damage / 5, xp);
    }

    #[test]
    fn test_xp_for_next() {
        let mut hero = new_char();
        assert_eq!(30, hero.xp_for_next());
        hero.increase_level();
        assert_eq!(84, hero.xp_for_next());
        hero.increase_level();
        assert_eq!(155, hero.xp_for_next());
    }

    #[test]
    fn test_add_experience() {
        let mut hero = new_char();
        assert_eq!(1, hero.level);
        assert_eq!(0, hero.xp);

        let level_up = hero.add_experience(20);
        assert!(!level_up);
        assert_eq!(1, hero.level);
        assert_eq!(20, hero.xp);

        let level_up = hero.add_experience(25);
        assert!(level_up);
        assert_eq!(2, hero.level);
        assert_eq!(15, hero.xp);
    }
}
