use crate::item::equipment;
use crate::item::equipment::Equipment;
use crate::location;
use crate::randomizer::{random, Randomizer};
use class::Class;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

pub mod class;

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Character {
    pub class: Class,
    pub sword: Option<equipment::Sword>,
    pub shield: Option<equipment::Shield>,

    pub level: i32,
    pub xp: i32,

    pub max_hp: i32,
    pub current_hp: i32,

    pub max_mp: i32,
    pub current_mp: i32,

    pub strength: i32,
    pub speed: i32,
    pub status_effect: Option<StatusEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StatusEffect {
    Burning,
    Poisoned,
}

pub struct Dead;
pub struct ClassNotFound;

impl Default for Character {
    fn default() -> Self {
        Character::player()
    }
}

impl Character {
    pub fn player() -> Self {
        Self::new(Class::player_default().clone(), 1)
    }

    pub fn enemy(level: i32, distance: location::Distance) -> Self {
        Self::new(Class::random_enemy(distance), level)
    }

    pub fn name(&self) -> String {
        self.class.name.to_string()
    }

    pub fn is_player(&self) -> bool {
        self.class.category == class::Category::Player
    }

    fn new(class: Class, level: i32) -> Self {
        // randomize level 1 stats by starting the increase from level 0
        let max_hp = class.hp.base() - class.hp.increase();
        let strength = class.strength.base() - class.strength.increase();
        let speed = class.speed.base() - class.speed.increase();
        let max_mp = class.mp.as_ref().map_or(0, |mp| mp.base() - mp.increase());

        let mut character = Self {
            class,
            sword: None,
            shield: None,
            level: 0,
            xp: 0,
            max_hp,
            current_hp: max_hp,
            max_mp,
            current_mp: max_mp,
            strength,
            speed,
            status_effect: None,
        };

        for _ in 0..level {
            character.increase_level();
        }

        character
    }

    /// Replace the character class with the one given by name.
    /// XP is lost. If the character is at level 1, it works as a re-roll
    /// with the new class; at other levels the initial stats are preserved.
    pub fn change_class(&mut self, name: &str) -> Result<i32, ClassNotFound> {
        if name == self.class.name {
            Ok(0)
        } else if let Some(class) = Class::player_class(name) {
            let lost_xp = self.xp;

            if self.level == 1 {
                // if class change is done at level 1, it works as a game reset
                // the player stats are regenerated with the new class
                // if equipment was already set, it is preserved
                let sword = self.sword.take();
                let shield = self.shield.take();
                *self = Self::new(class.clone(), 1);
                self.sword = sword;
                self.shield = shield;
            } else {
                self.class = class.clone();
            }

            self.xp = 0;
            Ok(lost_xp)
        } else {
            Err(ClassNotFound)
        }
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

    pub fn receive_damage(&mut self, damage: i32) -> Result<(), Dead> {
        if damage >= self.current_hp {
            self.current_hp = 0;
            Err(Dead)
        } else {
            self.current_hp -= damage;
            Ok(())
        }
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
    /// The second element is the mp cost of the attack, if any.
    pub fn damage(&self, receiver: &Self) -> (i32, i32) {
        let (damage, mp_cost) = if self.can_magic_attack() {
            (self.magic_attack(), self.mp_cost())
        } else {
            (self.physical_attack(), 0)
        };

        (max(1, damage - receiver.deffense()), mp_cost)
    }

    pub fn physical_attack(&self) -> i32 {
        if self.class.is_magic() {
            self.strength / 3
        } else {
            let sword_str = self.sword.as_ref().map_or(0, |s| s.strength());
            self.strength + sword_str
        }
    }

    pub fn magic_attack(&self) -> i32 {
        self.strength * 3
    }

    /// The character's class enables magic and there's enough mp left
    pub fn can_magic_attack(&self) -> bool {
        self.class.is_magic() && self.current_mp >= self.mp_cost()
    }

    fn mp_cost(&self) -> i32 {
        // each magic attack costs one third of the "canonical" mp total for this level
        self.class.mp.as_ref().map_or(0, |mp| mp.at(self.level) / 3)
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

    /// Return the status that this character's attack should inflict on the receiver.
    pub fn inflicted_status_effect(&self) -> Option<(StatusEffect, u32)> {
        // at some point the player could generate it depending on the equipment
        self.class.inflicts
    }

    pub fn maybe_remove_status_effect(&mut self) -> bool {
        if self.status_effect.is_some() {
            self.status_effect = None;
            return true;
        }
        false
    }

    /// If the character suffers from a damage-producing status effect, apply it.
    pub fn receive_status_effect_damage(&mut self) -> Result<Option<i32>, Dead> {
        // NOTE: in the future we could have a positive status that e.g. regen hp
        match self.status_effect {
            Some(StatusEffect::Burning) | Some(StatusEffect::Poisoned) => {
                let damage = std::cmp::max(1, self.max_hp / 20);
                let damage = random().damage(damage);
                self.receive_damage(damage)?;
                Ok(Some(damage))
            }
            _ => Ok(None),
        }
    }

    /// Return the player level rounded to offer items at "pretty levels", e.g.
    /// potion[1], sword[5]
    pub fn rounded_level(self: &Character) -> i32 {
        // allow level 1 or level 5n
        std::cmp::max(1, (self.level / 5) * 5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use class::Stat;

    fn new_char() -> Character {
        Character::new(
            Class {
                name: "test".to_string(),
                category: class::Category::Player,
                hp: Stat(25, 7),
                mp: None,
                strength: Stat(10, 3),
                speed: Stat(10, 2),
                inflicts: None,
            },
            1,
        )
    }

    #[test]
    fn test_new() {
        let hero = new_char();

        assert_eq!(1, hero.level);
        assert_eq!(0, hero.xp);

        assert_eq!(hero.class.hp.base(), hero.current_hp);
        assert_eq!(hero.class.hp.base(), hero.max_hp);
        assert_eq!(hero.class.strength.base(), hero.strength);
        assert_eq!(hero.class.speed.base(), hero.speed);
        assert!(hero.status_effect.is_none());
    }

    #[test]
    fn test_increase_level() {
        let mut hero = new_char();

        // assert what we're assuming are the params in the rest of the test
        assert_eq!(7, hero.class.hp.increase());
        assert_eq!(3, hero.class.strength.increase());
        assert_eq!(2, hero.class.speed.increase());

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
    fn test_receive_status_effect_damage() {
        let mut hero = new_char();
        assert_eq!(25, hero.current_hp);

        hero.receive_status_effect_damage().unwrap_or_default();
        assert_eq!(25, hero.current_hp);

        hero.status_effect = Some(StatusEffect::Burning);
        hero.receive_status_effect_damage().unwrap_or_default();
        assert_eq!(24, hero.current_hp);

        hero.status_effect = Some(StatusEffect::Poisoned);
        hero.receive_status_effect_damage().unwrap_or_default();
        assert_eq!(23, hero.current_hp);

        hero.maybe_remove_status_effect();
        hero.receive_status_effect_damage().unwrap_or_default();
        assert_eq!(23, hero.current_hp);

        hero.status_effect = Some(StatusEffect::Burning);
        hero.current_hp = 1;
        assert!(hero.receive_status_effect_damage().is_err());
        assert!(hero.is_dead());
    }

    #[test]
    fn test_class_change() {
        let mut player = Character::player();
        player.xp = 20;
        player.sword = Some(equipment::Sword::new(1));

        let warrior_class = Class::player_class("warrior").unwrap();
        let thief_class = Class::player_class("thief").unwrap();

        // attempt change to same class
        assert_eq!("warrior", player.class.name);
        assert!(player.change_class("warrior").is_ok());
        assert_eq!("warrior", player.class.name);
        assert_eq!(20, player.xp);
        assert_eq!(player.max_hp, warrior_class.hp.base());
        assert_eq!(player.strength, warrior_class.strength.base());
        assert_eq!(player.speed, warrior_class.speed.base());
        assert!(player.sword.is_some());

        // attempt change to unknown class
        assert!(player.change_class("choripan").is_err());

        // attempt change to different class at level 1 (reset)
        assert!(player.change_class("thief").is_ok());
        assert_eq!("thief", player.class.name);
        assert_eq!(0, player.xp);
        assert_eq!(player.max_hp, thief_class.hp.base());
        assert_eq!(player.strength, thief_class.strength.base());
        assert_eq!(player.speed, thief_class.speed.base());
        assert!(player.sword.is_some());

        // attempt change to different class at level 2
        player.level = 2;
        player.xp = 20;
        assert!(player.change_class("warrior").is_ok());
        assert_eq!("warrior", player.class.name);
        assert_eq!(0, player.xp);
        assert_eq!(player.max_hp, thief_class.hp.base());
        assert_eq!(player.strength, thief_class.strength.base());
        assert_eq!(player.speed, thief_class.speed.base());
        assert!(player.sword.is_some());
    }
}
