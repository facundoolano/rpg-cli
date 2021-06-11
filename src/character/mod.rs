use crate::game::battle::Attack;
use crate::item::equipment;
use crate::item::equipment::Equipment;
use crate::location;
use crate::randomizer::{random, Randomizer};
use class::Class;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

pub mod class;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum StatusEffect {
    Normal,
    Burned(i32),
    Poisoned(i32),
    Confused,
}

impl StatusEffect {
    pub fn is_normal(&self) -> bool {
        matches!(self, StatusEffect::Normal)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    #[serde(skip, default = "default_class")]
    class: &'static Class,
    pub sword: Option<equipment::Sword>,
    pub shield: Option<equipment::Shield>,

    pub level: i32,
    pub xp: i32,

    pub max_hp: i32,
    pub current_hp: i32,

    pub strength: i32,
    pub speed: i32,
    pub status_effect: StatusEffect,
}

// Always attach the static hero class to deserialized characters
fn default_class() -> &'static Class {
    &Class::HERO
}

impl Character {
    pub fn player() -> Self {
        Self::new(&Class::HERO, 1, None, None)
    }

    pub fn enemy(level: i32, distance: location::Distance) -> Self {
        Self::new(Class::random_enemy(distance), level, None, None)
    }

    pub fn ascend(class: &'static Class, level: i32, sword: Option<equipment::Sword>,
                   shield: Option<equipment::Shield>) -> Self {
        Self::new(class, level, sword, shield)
    }

    pub fn name(&self) -> String {
        self.class.name.to_string()
    }

    pub fn is_player(&self) -> bool {
        // kind of ugly but does the job
        self.class.name == "hero"
    }

    fn new(class: &'static Class, level: i32, sword: Option<equipment::Sword>,
           shield: Option<equipment::Shield>) -> Self {
        let mut character = Self {
            class,
            sword: sword,
            shield: shield,
            level: 1,
            xp: 0,
            max_hp: class.hp.base(),
            current_hp: class.hp.base(),
            strength: class.strength.base(),
            speed: class.speed.base(),
            status_effect: StatusEffect::Normal,
        };

        for _ in 1..level {
            character.increase_level();
        }

        character
    }

    /// Raise the level and all the character stats.
    fn increase_level(&mut self) {
        self.level += 1;

        self.strength += random().stat_increase(self.class.strength.increase());
        self.speed += random().stat_increase(self.class.speed.increase());

        // the current should increase proportionally but not
        // erase previous damage
        let previous_damage = self.max_hp - self.current_hp;
        self.max_hp += random().stat_increase(self.class.hp.increase());
        self.current_hp = self.max_hp - previous_damage;
    }

    /// Add to the accumulated experience points, possibly increasing the level.
    pub fn add_experience(&mut self, xp: i32) -> i32 {
        self.xp += xp;

        let mut increased_levels = 0;
        let mut for_next = self.xp_for_next();
        while self.xp >= for_next {
            self.increase_level();
            self.xp -= for_next;
            increased_levels += 1;
            for_next = self.xp_for_next();
        }
        increased_levels
    }

    pub fn receive_damage(&mut self, damage: i32) {
        self.current_hp = max(0, self.current_hp - damage);
    }

    pub fn is_dead(&self) -> bool {
        self.current_hp == 0
    }

    /// Restore up to the given amount of health points (not exceeding the max_hp).
    /// Return the amount actually restored.
    pub fn heal(&mut self, amount: i32) -> i32 {
        let previous = self.current_hp;
        self.current_hp = min(self.max_hp, self.current_hp + amount);
        self.current_hp - previous
    }

    /// Restore all health points to the max_hp
    pub fn heal_full(&mut self) -> i32 {
        self.heal(self.max_hp)
    }

    /// How many experience points are required to move to the next level.
    pub fn xp_for_next(&self) -> i32 {
        let exp = 1.5;
        let base_xp = 30.0;
        (base_xp * (self.level as f64).powf(exp)) as i32
    }

    /// Generate a randomized damage number based on the attacker strength
    /// and the receiver strength.
    pub fn damage(&self, receiver: &Self) -> i32 {
        max(1, self.attack() - receiver.deffense())
    }

    pub fn attack(&self) -> i32 {
        let sword_str = self.sword.as_ref().map_or(0, |s| s.strength());
        self.strength + sword_str
    }

    pub fn deffense(&self) -> i32 {
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

    pub fn receive_status_effect(&mut self, status_effect: StatusEffect) {
        self.status_effect = status_effect;
    }

    pub fn maybe_remove_status_effect(&mut self) -> bool {
        if !self.status_effect.is_normal() {
            self.receive_status_effect(StatusEffect::Normal);
            return true;
        }

        false
    }

    pub fn maybe_receive_status_effect(&mut self) -> bool {
        if !self.is_dead() && self.status_effect.is_normal() {
            let status_effect = random().status_effect();
            if !status_effect.is_normal() {
                self.receive_status_effect(status_effect);
                return true;
            }
        }

        false
    }

    pub fn apply_status_effect(&mut self) -> Attack {
        match self.status_effect {
            StatusEffect::Burned(damage) | StatusEffect::Poisoned(damage) => {
                self.receive_damage(damage);
                Attack::Effect(self.status_effect)
            }
            StatusEffect::Normal | StatusEffect::Confused => Attack::Miss,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use class::Stat;

    const TEST_CLASS: Class = Class {
        name: "test",
        hp: Stat(25, 7),
        strength: Stat(10, 3),
        speed: Stat(10, 2),
    };

    fn new_char() -> Character {
        Character::new(&TEST_CLASS, 1, None, None)
    }

    #[test]
    fn test_new() {
        let hero = new_char();

        assert_eq!(1, hero.level);
        assert_eq!(0, hero.xp);

        assert_eq!(TEST_CLASS.hp.base(), hero.current_hp);
        assert_eq!(TEST_CLASS.hp.base(), hero.max_hp);
        assert_eq!(TEST_CLASS.strength.base(), hero.strength);
        assert_eq!(TEST_CLASS.speed.base(), hero.speed);
        assert!(matches!(hero.status_effect, StatusEffect::Normal));
    }

    #[test]
    fn test_increase_level() {
        let mut hero = new_char();

        // assert what we're assuming are the params in the rest of the test
        assert_eq!(7, TEST_CLASS.hp.increase());
        assert_eq!(3, TEST_CLASS.strength.increase());
        assert_eq!(2, TEST_CLASS.speed.increase());

        hero.max_hp = 20;
        hero.current_hp = 20;
        hero.strength = 10;
        hero.speed = 5;

        hero.increase_level();
        assert_eq!(2, hero.level);
        assert_eq!(27, hero.max_hp);
        assert_eq!(13, hero.strength);
        assert_eq!(7, hero.speed);

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

        assert_eq!(0, hero.add_experience(20));
        assert_eq!(1, hero.level);
        assert_eq!(20, hero.xp);

        assert_eq!(1, hero.add_experience(25));
        assert_eq!(2, hero.level);
        assert_eq!(15, hero.xp);

        // multiple increases at once
        let mut hero = new_char();
        assert_eq!(2, hero.add_experience(120));
        assert!(hero.xp < hero.xp_for_next());
        assert_eq!(3, hero.level);
        assert_eq!(6, hero.xp);
    }

    #[test]
    fn test_heal() {
        let mut hero = new_char();
        assert_eq!(25, hero.max_hp);
        assert_eq!(25, hero.current_hp);

        assert_eq!(0, hero.heal(100));
        assert_eq!(25, hero.max_hp);
        assert_eq!(25, hero.current_hp);

        assert_eq!(0, hero.heal_full());
        assert_eq!(25, hero.max_hp);
        assert_eq!(25, hero.current_hp);

        hero.current_hp = 10;
        assert_eq!(5, hero.heal(5));
        assert_eq!(25, hero.max_hp);
        assert_eq!(15, hero.current_hp);

        assert_eq!(10, hero.heal(100));
        assert_eq!(25, hero.max_hp);
        assert_eq!(25, hero.current_hp);

        hero.current_hp = 10;
        assert_eq!(15, hero.heal_full());
        assert_eq!(25, hero.max_hp);
        assert_eq!(25, hero.current_hp);
    }

    #[test]
    fn test_overflow() {
        let mut hero = Character::player();

        while hero.level < 500 {
            hero.add_experience(hero.xp_for_next());
            hero.sword = Some(equipment::Sword::new(hero.level));
            let turns_unarmed = hero.max_hp / hero.strength;
            let turns_armed = hero.max_hp / hero.attack();
            println!(
                "hero[{}] next={} hp={} spd={} str={} att={} turns_u={} turns_a={}",
                hero.level,
                hero.xp_for_next(),
                hero.max_hp,
                hero.speed,
                hero.strength,
                hero.attack(),
                turns_unarmed,
                turns_armed
            );

            assert!(hero.max_hp > 0);
            assert!(hero.speed > 0);
            assert!(hero.attack() > 0);

            assert!(turns_armed < turns_unarmed);
            assert!(turns_armed < 20);
        }
        // assert!(false);
    }

    #[test]
    fn test_receive_status_effect() {
        let mut hero = Character::player();

        hero.maybe_receive_status_effect();
        assert!(matches!(hero.status_effect, StatusEffect::Normal));
    }
}
