use crate::item::equipment;
use crate::item::key::Key;
use crate::item::ring::Ring;
use crate::item::Item;
use crate::log;
use crate::randomizer::{random, Randomizer};
use class::Class;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

pub mod class;
pub mod enemy;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Character {
    pub class: Class,
    pub level: i32,
    pub xp: i32,

    max_hp: i32,
    pub current_hp: i32,

    max_mp: i32,
    pub current_mp: i32,

    strength: i32,
    speed: i32,

    pub sword: Option<equipment::Equipment>,
    pub shield: Option<equipment::Equipment>,
    pub left_ring: Option<Ring>,
    pub right_ring: Option<Ring>,

    pub status_effect: Option<StatusEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StatusEffect {
    Burn,
    Poison,
}

/// Outcome of an attack attempt.
/// This affects primarily how the attack is displayed.
pub enum AttackType {
    Regular,
    Critical,
    Effect(StatusEffect),
    Miss,
}

#[derive(Debug)]
pub struct Dead;
pub struct ClassNotFound;

impl Default for Character {
    fn default() -> Self {
        Character::player()
    }
}

impl Character {
    pub fn player() -> Self {
        Self::new(Class::player_first().clone(), 1)
    }

    pub fn name(&self) -> String {
        self.class.name.to_string()
    }

    pub fn is_player(&self) -> bool {
        self.class.category == class::Category::Player
    }

    pub fn new(class: Class, level: i32) -> Self {
        let max_hp = class.hp.base();
        let strength = class.strength.base();
        let speed = class.speed.base();
        let max_mp = class.mp.as_ref().map_or(0, |mp| mp.base());

        let mut character = Self {
            class,
            sword: None,
            shield: None,
            left_ring: None,
            right_ring: None,
            level: 1,
            xp: 0,
            max_hp,
            current_hp: max_hp,
            max_mp,
            current_mp: max_mp,
            strength,
            speed,
            status_effect: None,
        };

        for _ in 1..level {
            character.raise_level();
        }

        character
    }

    /// Replace the character class with the one given by name.
    /// XP is lost. If the character is at level 1, it works as a re-roll
    /// with the new class; at other levels the initial stats are preserved.
    pub fn change_class(&mut self, name: &str) -> Result<(), ClassNotFound> {
        if name == self.class.name {
            Ok(())
        } else if let Some(class) = Class::player_by_name(name) {
            let lost_xp = self.xp;

            if self.level == 1 {
                // if class change is done at level 1, it works as a game reset
                // the player stats are regenerated with the new class
                // if equipment was already set, it is preserved
                let sword = self.sword.take();
                let shield = self.shield.take();
                let left_ring = self.left_ring.take();
                let right_ring = self.right_ring.take();

                *self = Self::new(class.clone(), 1);
                self.sword = sword;
                self.shield = shield;
                self.left_ring = left_ring;
                self.right_ring = right_ring;
            } else {
                self.class = class.clone();

                // if switching to a magic class on a higher level, we need to
                // force the base mp so it can attack like a level 1 char
                // rather than having no magic at all
                if class.is_magic() && self.max_mp == 0 {
                    let base_mp = class.mp.as_ref().map(|mp| mp.base()).unwrap();
                    self.max_mp = base_mp;
                    self.current_mp = base_mp;
                }
            }

            self.xp = 0;
            log::change_class(self, lost_xp);
            Ok(())
        } else {
            Err(ClassNotFound)
        }
    }

    /// Raise the level and all the character stats.
    pub fn raise_level(&mut self) {
        self.level += 1;
        self.raise_strength();
        self.raise_speed();
        self.raise_hp();
        self.raise_mp();
    }

    pub fn raise_strength(&mut self) -> i32 {
        let inc = self.class.strength.increase();
        self.strength += inc;
        inc
    }

    pub fn raise_speed(&mut self) -> i32 {
        let inc = self.class.speed.increase();
        self.speed += inc;
        inc
    }

    pub fn raise_hp(&mut self) -> i32 {
        // the current should increase proportionally but not
        // erase previous damage
        let previous_damage = self.max_hp() - self.current_hp;
        let inc = self.class.hp.increase();
        self.max_hp += inc;
        self.current_hp = self.max_hp() - previous_damage;
        inc
    }

    pub fn raise_mp(&mut self) -> i32 {
        // the current should increase proportionally but not
        // erase previous mp consumption
        let previous_used_mp = self.max_mp() - self.current_mp;
        let inc = self.class.mp.as_ref().map_or(0, |mp| mp.increase());
        self.max_mp += inc;
        self.current_mp = self.max_mp() - previous_used_mp;
        inc
    }

    /// Add to the accumulated experience points, possibly increasing the level.
    pub fn add_experience(&mut self, xp: i32) -> i32 {
        self.xp += xp;

        let mut increased_levels = 0;
        let mut for_next = self.xp_for_next();
        while self.xp >= for_next {
            self.raise_level();
            self.xp -= for_next;
            increased_levels += 1;
            for_next = self.xp_for_next();
        }
        increased_levels
    }

    /// Add or subtract the given amount of current hp, keeping it between
    /// 0 and max_hp. Return the effectively changed amount, or Err(Dead)
    /// if the character dies as a consequence of the damage.
    pub fn update_hp(&mut self, amount: i32) -> Result<i32, Dead> {
        let previous = self.current_hp;
        self.current_hp = max(0, min(self.max_hp(), self.current_hp + amount));

        if self.current_hp == 0 {
            Err(Dead)
        } else {
            Ok(self.current_hp - previous)
        }
    }

    /// Add or subtract the given amount of current mp, keeping it between
    /// 0 and max_mp.
    pub fn update_mp(&mut self, amount: i32) -> i32 {
        let previous = self.current_mp;
        self.current_mp = max(0, min(self.max_mp(), self.current_mp + amount));
        self.current_mp - previous
    }

    /// Restore all health and magic points to their max and remove status effects
    pub fn restore(&mut self) -> (i32, i32, bool) {
        let healed = self.status_effect.is_some();
        self.status_effect = None;
        (
            self.update_hp(self.max_hp()).unwrap(),
            self.update_mp(self.max_mp()),
            healed,
        )
    }

    /// How many experience points are required to move to the next level.
    pub fn xp_for_next(&self) -> i32 {
        let exp = 1.5;
        let base_xp = 30.0;
        (base_xp * (self.level as f64).powf(exp)) as i32
    }

    pub fn max_hp(&self) -> i32 {
        self.modify_stat(self.max_hp, Ring::HP)
    }

    pub fn max_mp(&self) -> i32 {
        self.modify_stat(self.max_mp, Ring::MP)
    }

    pub fn speed(&self) -> i32 {
        self.modify_stat(self.speed, Ring::Speed)
    }

    /// Generate and log an attack of this character and apply its effects to
    /// the given receiver.
    /// Returns a tuple with the gained experience and a Err(Dead) result if
    /// the receiver died from the inflicted damage.
    pub fn attack(&mut self, receiver: &mut Self) -> (i32, Result<(), Dead>) {
        let (damage, mp_cost) = self.damage(receiver);
        let damage = random().damage(damage);
        let xp = self.xp_gained(receiver, damage);

        let attack_type = self.attack_type(receiver);
        let (damage, xp) = match attack_type {
            AttackType::Regular => (damage, xp),
            AttackType::Critical => (damage * 2, xp),
            AttackType::Effect(_) => (damage, xp),
            AttackType::Miss => (0, 0),
        };

        self.update_mp(-mp_cost);

        // The receiver can die from the damage. Return the result for
        // the caller to handle that scenario.
        let result = receiver.update_hp(-damage).map(|_| ());
        if let AttackType::Effect(status) = attack_type {
            receiver.status_effect = Some(status);
        }

        log::attack(receiver, &attack_type, damage, mp_cost);

        (xp, result)
    }

    /// If the double beat ring is equipped, attack the receiver.
    pub fn maybe_double_beat(&mut self, receiver: &mut Self) {
        if receiver.current_hp > 0
            && (self.left_ring == Some(Ring::Double) || self.right_ring == Some(Ring::Double))
        {
            // assuming it's always the player and we don't need to handle death
            let _ = self.attack(receiver);
        }
    }

    /// If the counter attack ring is equipped randomly counter attack the receiver.
    pub fn maybe_counter_attack(&mut self, receiver: &mut Self) {
        let wearing_counter =
            self.left_ring == Some(Ring::Counter) || self.right_ring == Some(Ring::Counter);
        if wearing_counter && random().counter_attack() {
            // assuming it's always the player and we don't need to handle death
            let _ = self.attack(receiver);
        }
    }

    /// If the revive ring is equipped and the character died, restore 10% of its hp.
    /// Intended to be used once per battle, with `already_revived` tracking whether
    /// it was used before.
    /// Returns Err(Dead) if can't be recovered from death, otherwise Ok(already_revived).
    pub fn maybe_revive(
        &mut self,
        died: Result<(), Dead>,
        already_revived: bool,
    ) -> Result<bool, Dead> {
        let wearing_revive =
            self.left_ring == Some(Ring::Revive) || self.right_ring == Some(Ring::Revive);
        match died {
            Ok(()) => Ok(already_revived),
            Err(Dead) if wearing_revive && !already_revived => {
                let restored = max(1, self.max_hp() / 10);
                self.current_hp = restored;
                log::heal_item(self, "revive", restored, 0, false);
                Ok(true)
            }
            Err(Dead) => Err(Dead),
        }
    }

    /// Generate a randomized regular/miss/critical/status effect attack based
    /// on the stats of both characters.
    fn attack_type(&self, receiver: &Self) -> AttackType {
        let inflicted_status = random().inflicted(self.inflicted_status_effect(receiver));

        if random().is_miss(self.speed(), receiver.speed()) {
            AttackType::Miss
        } else if random().is_critical() {
            AttackType::Critical
        } else if let Some(status) = inflicted_status {
            AttackType::Effect(status)
        } else {
            AttackType::Regular
        }
    }

    /// Generate a damage number based on the attacker strength and the receiver
    /// deffense.
    /// The second element is the mp cost of the attack, if any.
    pub fn damage(&self, receiver: &Self) -> (i32, i32) {
        let (damage, mp_cost) = if self.can_magic_attack() {
            (self.magic_attack(), self.attack_mp_cost())
        } else {
            (self.physical_attack(), 0)
        };

        (max(1, damage - receiver.deffense()), mp_cost)
    }

    /// The character's class enables magic and there's enough mp left
    pub fn can_magic_attack(&self) -> bool {
        self.class.is_magic() && self.current_mp >= self.attack_mp_cost()
    }

    fn attack_mp_cost(&self) -> i32 {
        // each magic attack costs one third of the "canonical" mp total for this level
        self.class.mp.as_ref().map_or(0, |mp| mp.at(self.level) / 3)
    }

    /// Amount of damage the character can inflict with physical atacks, given
    /// its strength and equipment. Magic using characters' strength is dimmed.
    pub fn physical_attack(&self) -> i32 {
        let sword_str = self.sword.as_ref().map_or(0, |s| s.strength());
        let attack = self.modify_stat(self.strength, Ring::Attack) + sword_str;
        if self.class.is_magic() {
            attack / 3
        } else {
            attack
        }
    }

    /// Amount of damage the character can inflict with magical attacks.
    /// Zero if the current character class is not magic.
    pub fn magic_attack(&self) -> i32 {
        if self.class.is_magic() {
            let base = self.strength * 3;
            self.modify_stat(base, Ring::Magic)
        } else {
            0
        }
    }

    pub fn deffense(&self) -> i32 {
        let shield_str = self.shield.as_ref().map_or(0, |s| s.strength());
        // base strength should be zero, subtract it from ring calculation
        shield_str + self.modify_stat(self.strength, Ring::Deffense) - self.strength
    }

    /// How many experience points are gained by inflicting damage to an enemy.
    fn xp_gained(&self, receiver: &Self, damage: i32) -> i32 {
        let class_multiplier = match receiver.class.category {
            class::Category::Rare => 3,
            class::Category::Legendary => 5,
            _ => 1,
        };

        // don't consider xp beyond the actually inflicted damage, otherwise
        // the stronger the char, the more xp even if defeating a weak enemy.
        let damage = min(damage, receiver.current_hp);

        if self.level > receiver.level + 10 {
            // don't reward cheap victories
            0
        } else if receiver.level > self.level {
            damage * (1 + receiver.level - self.level) * class_multiplier
        } else {
            damage / (1 + self.level - receiver.level) * class_multiplier
        }
    }

    /// Return the status that this character's attack should inflict on the receiver.
    fn inflicted_status_effect(&self, receiver: &Self) -> Option<(StatusEffect, u32)> {
        if receiver.left_ring == Some(Ring::Protect) || receiver.right_ring == Some(Ring::Protect) {
            return None;
        }

        let ring_status = match (self.left_ring.as_ref(), self.right_ring.as_ref()) {
            (Some(Ring::Poison), _) | (_, Some(Ring::Poison)) => Some((StatusEffect::Poison, 3)),
            (Some(Ring::Fire), _) | (_, Some(Ring::Fire)) => Some((StatusEffect::Burn, 3)),
            _ => None,
        };

        let result = self.class.inflicts.or(ring_status);
        if let Some((status, _)) = result {
            // don't double-inflict if already has the same status
            if receiver.status_effect == Some(status) {
                return None;
            }
        }
        result
    }

    /// If the character has a status condition (e.g. poison) or an equipped
    /// ring that produces one (e.g. regen hp), apply its effects.
    pub fn apply_status_effects(&mut self) -> Result<(), Dead> {
        let mut hp_effect = 0;
        let mut mp_effect = 0;

        // statuses have a (randomized) +/-5% effect on the base stat
        let hp_unit = || random().damage(std::cmp::max(1, self.max_hp / 20));
        let mp_unit = || random().damage(std::cmp::max(1, self.max_mp / 20));

        if self.left_ring == Some(Ring::RegenHP) || self.right_ring == Some(Ring::RegenHP) {
            hp_effect += hp_unit();
        }

        if self.class.is_magic()
            && (self.left_ring == Some(Ring::RegenMP) || self.right_ring == Some(Ring::RegenMP))
        {
            mp_effect += mp_unit();
        }

        if self.left_ring == Some(Ring::Ruling) || self.right_ring == Some(Ring::Ruling) {
            hp_effect -= hp_unit();
        }

        if self.status_effect == Some(StatusEffect::Burn)
            || self.status_effect == Some(StatusEffect::Poison)
        {
            hp_effect -= hp_unit();
        }

        let result = self.update_hp(hp_effect).map(|_| ());
        self.update_mp(mp_effect);

        log::status_effect(self, hp_effect, mp_effect);

        result
    }

    /// Return the player level rounded to offer items at "pretty levels", e.g.
    /// potion[1], sword[5]
    pub fn rounded_level(self: &Character) -> i32 {
        // allow level 1 or level 5n
        std::cmp::max(1, (self.level / 5) * 5)
    }

    /// Equip the given ring and apply its side-effects.
    /// If already carrying two rings, the least recently equipped one is
    /// removed, undoing its side-effects.
    pub fn equip_ring(&mut self, ring: Ring) -> Option<Ring> {
        let removed = self.right_ring.take();
        self.unequip_ring_side_effect(&removed);
        self.equip_ring_side_effect(&ring);
        self.right_ring = self.left_ring.replace(ring);

        removed
    }

    /// Remove the ring by the given name from the equipment (if any),
    /// unapplying its side-effects.
    pub fn unequip_ring(&mut self, name: &Key) -> Option<Ring> {
        match (self.left_ring.clone(), self.right_ring.clone()) {
            (Some(ring), _) if ring.key() == *name => {
                let removed = self.left_ring.take();
                self.unequip_ring_side_effect(&removed);
                self.left_ring = self.right_ring.take();
                removed
            }
            (_, Some(ring)) if ring.key() == *name => {
                let removed = self.right_ring.take();
                self.unequip_ring_side_effect(&removed);
                removed
            }
            _ => None,
        }
    }

    /// Return true if an evade ring is equipped, i.e. no enemies should appear.
    pub fn enemies_evaded(&self) -> bool {
        self.left_ring == Some(Ring::Evade) || self.right_ring == Some(Ring::Evade)
    }

    /// Return true if a chest ring is equipped, i.e. the character should have double
    /// chance to find a chest.
    pub fn double_chests(&self) -> bool {
        self.left_ring == Some(Ring::Chest) || self.right_ring == Some(Ring::Chest)
    }

    /// Return the gold that should be rewarded for beating an enemy of the given
    /// level. Doubled if the gold ring is equipped.
    pub fn gold_gained(&self, enemy_level: i32) -> i32 {
        let level = max(1, enemy_level - self.level);
        let gold = random().gold_gained(level * 50);

        if self.level > enemy_level + 10 {
            // don't reward cheap victories
            0
        } else if self.left_ring == Some(Ring::Gold) || self.right_ring == Some(Ring::Gold) {
            gold * 2
        } else {
            gold
        }
    }

    /// Apply any side-effects of the ring over the character stats, e.g.
    /// increasing its max hp for an HP ring.
    fn equip_ring_side_effect(&mut self, ring: &Ring) {
        match ring {
            Ring::HP => {
                self.current_hp += (ring.factor() * self.max_hp as f64) as i32;
            }
            Ring::MP => {
                self.current_mp += (ring.factor() * self.max_mp as f64) as i32;
            }
            _ => {}
        }
    }

    /// Unapply the side-effects of the ring on the character.
    fn unequip_ring_side_effect(&mut self, ring: &Option<Ring>) {
        match ring {
            Some(Ring::HP) => {
                let to_remove = (ring.as_ref().unwrap().factor() * self.max_hp as f64) as i32;
                self.current_hp = max(1, self.current_hp - to_remove);
            }
            Some(Ring::MP) => {
                let to_remove = (ring.as_ref().unwrap().factor() * self.max_mp as f64) as i32;
                self.current_mp = max(1, self.current_mp - to_remove);
            }
            _ => {}
        }
    }

    /// If either ring matches the given one, apply the ring effect
    /// to the given base stat, e.g. for an HP ring increase the base HP.
    fn modify_stat(&self, base: i32, ring: Ring) -> i32 {
        let mut factor = 1.0;
        if self.left_ring.as_ref() == Some(&ring) {
            factor += ring.factor();
        }
        if self.right_ring.as_ref() == Some(&ring) {
            factor += ring.factor();
        }
        (base as f64 * factor).round() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use class::Stat;

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

        hero.raise_level();
        assert_eq!(2, hero.level);
        assert_eq!(27, hero.max_hp);
        assert_eq!(13, hero.strength);
        assert_eq!(7, hero.speed);

        let damage = 7;
        hero.current_hp -= damage;

        hero.raise_level();
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
        assert_eq!(10, hero.damage(&foe).0);

        // level 1 vs level 2
        foe.level = 2;
        foe.strength = 15;
        assert_eq!(10, hero.damage(&foe).0);

        // level 2 vs level 1
        assert_eq!(15, foe.damage(&hero).0);

        // level 1 vs level 5
        foe.level = 5;
        foe.strength = 40;
        assert_eq!(10, hero.damage(&foe).0);

        // level 5 vs level 1
        assert_eq!(40, foe.damage(&hero).0);
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
        hero.raise_level();
        assert_eq!(84, hero.xp_for_next());
        hero.raise_level();
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

        assert_eq!(0, hero.update_hp(100).unwrap());
        assert_eq!(25, hero.max_hp);
        assert_eq!(25, hero.current_hp);

        assert_eq!(0, hero.restore().0);
        assert_eq!(25, hero.max_hp);
        assert_eq!(25, hero.current_hp);

        hero.current_hp = 10;
        assert_eq!(5, hero.update_hp(5).unwrap());
        assert_eq!(25, hero.max_hp);
        assert_eq!(15, hero.current_hp);

        assert_eq!(10, hero.update_hp(100).unwrap());
        assert_eq!(25, hero.max_hp);
        assert_eq!(25, hero.current_hp);

        hero.current_hp = 10;
        assert_eq!(15, hero.restore().0);
        assert_eq!(25, hero.max_hp);
        assert_eq!(25, hero.current_hp);
    }

    #[test]
    fn test_overflow() {
        let mut hero = Character::player();

        while hero.level < 500 {
            hero.add_experience(hero.xp_for_next());
            hero.sword = Some(equipment::Equipment::sword(hero.level));
            let turns_unarmed = hero.max_hp / hero.strength;
            let turns_armed = hero.max_hp / hero.physical_attack();
            println!(
                "hero[{}] next={} hp={} spd={} str={} att={} turns_u={} turns_a={}",
                hero.level,
                hero.xp_for_next(),
                hero.max_hp,
                hero.speed,
                hero.strength,
                hero.physical_attack(),
                turns_unarmed,
                turns_armed
            );

            assert!(hero.max_hp > 0);
            assert!(hero.speed > 0);
            assert!(hero.physical_attack() > 0);

            assert!(turns_armed < turns_unarmed);
            assert!(turns_armed < 20);
        }
        // assert!(false);
    }

    #[test]
    fn apply_status_effect() {
        let mut hero = new_char();
        assert_eq!(25, hero.current_hp);

        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(25, hero.current_hp);

        hero.status_effect = Some(StatusEffect::Burn);
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(24, hero.current_hp);

        hero.status_effect = Some(StatusEffect::Poison);
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(23, hero.current_hp);

        hero.status_effect = None;
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(23, hero.current_hp);

        hero.status_effect = Some(StatusEffect::Burn);
        hero.current_hp = 1;
        assert!(hero.apply_status_effects().is_err());
        assert_eq!(0, hero.current_hp);
    }

    #[test]
    fn apply_ring_status() {
        let mut hero = new_char();
        assert_eq!(25, hero.current_hp);

        // hp ring already full
        hero.left_ring = Some(Ring::RegenHP);
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(25, hero.current_hp);

        // hp ring recover
        hero.current_hp = 20;
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(21, hero.current_hp);

        // mp ring non magic
        hero.left_ring = Some(Ring::RegenMP);
        assert_eq!(0, hero.current_mp);
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(0, hero.current_mp);

        // force into a magic class
        hero.class.mp = Some(class::Stat(10, 1));
        hero.max_mp = 10;
        hero.current_mp = 10;

        // mp ring magic already full
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(10, hero.current_mp);

        // mp ring magic recover
        hero.current_mp = 7;
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(8, hero.current_mp);

        // hp + mp
        hero.right_ring = Some(Ring::RegenHP);
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(22, hero.current_hp);
        assert_eq!(9, hero.current_mp);

        // mp + hp
        hero.left_ring = Some(Ring::RegenHP);
        hero.right_ring = Some(Ring::RegenMP);
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(23, hero.current_hp);
        assert_eq!(10, hero.current_mp);

        // hp - burn cancel each other
        hero.right_ring = None;
        hero.status_effect = Some(StatusEffect::Burn);
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(23, hero.current_hp);
        assert_eq!(10, hero.current_mp);

        // hp - burn prevent dead
        hero.current_hp = 1;
        hero.apply_status_effects().unwrap_or_default();
        assert_eq!(1, hero.current_hp);
    }

    #[test]
    fn test_class_change() {
        let mut player = Character::player();
        player.xp = 20;
        player.sword = Some(equipment::Equipment::sword(1));

        let warrior_class = Class::player_by_name("warrior").unwrap();
        let thief_class = Class::player_by_name("thief").unwrap();

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

    #[test]
    fn test_change_to_magic_class() {
        let mut player = Character::player();
        assert_eq!("warrior", player.class.name);
        assert_eq!(0, player.max_mp);
        assert_eq!(0, player.current_mp);

        // when changing at level 1, it's a re-roll of the character
        player.change_class("mage").unwrap_or_default();
        let base_mp = player.class.mp.as_ref().map_or(0, |mp| mp.base());
        assert!(base_mp > 0);
        assert_eq!(base_mp, player.max_mp);
        assert_eq!(base_mp, player.current_mp);

        player.change_class("warrior").unwrap_or_default();
        assert_eq!(0, player.max_mp);
        assert_eq!(0, player.current_mp);

        player.raise_level();
        player.raise_level();
        assert_eq!(0, player.max_mp);
        assert_eq!(0, player.current_mp);

        // in level > 1, change to magic class should give base magic instead of zero
        player.change_class("mage").unwrap_or_default();
        assert_eq!(base_mp, player.max_mp);
        assert_eq!(base_mp, player.current_mp);
    }

    #[test]
    fn test_magic_attacks() {
        let mut hero = Character::player();
        let foe = new_char();

        assert_eq!("warrior", hero.class.name);
        assert!(!hero.can_magic_attack());
        let base_strength = hero.class.strength.base();

        // warrior mp = 0
        assert_eq!((base_strength, 0), hero.damage(&foe));

        // warrior with non zero mp, mp = 0
        // (this can happen if accumulated mp via class change)
        hero.current_mp = 10;
        hero.max_mp = 10;
        assert!(!hero.can_magic_attack());
        assert_eq!((base_strength, 0), hero.damage(&foe));

        // warrior + sword, increased damage + mp = 0
        let sword = equipment::Equipment::sword(hero.level);
        let sword_strength = sword.strength();
        hero.sword = Some(sword);
        assert_eq!((base_strength + sword_strength, 0), hero.damage(&foe));

        let mut mage = Character::player();
        mage.change_class("mage").unwrap_or_default();
        assert_eq!("mage", mage.class.name);
        assert!(mage.can_magic_attack());

        // mage with enough mp, -mp, *3
        let base_strength = mage.class.strength.base();
        assert_eq!((base_strength * 3, mage.max_mp / 3), mage.damage(&foe));

        // enough for one more
        mage.current_mp = mage.max_mp / 3;
        assert!(mage.can_magic_attack());
        assert_eq!((base_strength * 3, mage.max_mp / 3), mage.damage(&foe));

        // with sword, it affects the physical attacks
        mage.sword = Some(equipment::Equipment::sword(hero.level));
        assert_eq!((base_strength * 3, mage.max_mp / 3), mage.damage(&foe));

        // mage without enough mp, 0 mp, /3
        mage.current_mp = mage.max_mp / 3 - 1;
        assert!(!mage.can_magic_attack());
        assert_eq!(((base_strength + sword_strength) / 3, 0), mage.damage(&foe));
    }

    #[test]
    fn test_hp_ring() {
        let mut char = new_plain_stats_char();
        assert_eq!(10, char.current_hp);
        assert_eq!(10, char.max_hp());

        char.equip_ring(Ring::HP);
        assert_eq!(15, char.max_hp());
        assert_eq!(15, char.current_hp);

        char.equip_ring(Ring::HP);
        assert_eq!(20, char.max_hp());
        assert_eq!(20, char.current_hp);

        // push out to unequip
        char.unequip_ring(&Key::Ring(Ring::HP));
        assert_eq!(15, char.max_hp());
        assert_eq!(15, char.current_hp);

        char.unequip_ring(&Key::Ring(Ring::HP));
        assert_eq!(10, char.max_hp());
        assert_eq!(10, char.current_hp);

        // preserve taken damage
        char.current_hp -= 3;

        char.equip_ring(Ring::HP);
        assert_eq!(15, char.max_hp());
        assert_eq!(12, char.current_hp);

        char.unequip_ring(&Key::Ring(Ring::HP));
        assert_eq!(10, char.max_hp());
        assert_eq!(7, char.current_hp);
    }

    #[test]
    fn test_mp_ring() {
        let mut char = new_plain_stats_char();
        assert_eq!(10, char.current_mp);
        assert_eq!(10, char.max_mp());

        char.equip_ring(Ring::MP);
        assert_eq!(15, char.max_mp());
        assert_eq!(15, char.current_mp);

        char.equip_ring(Ring::MP);
        assert_eq!(20, char.max_mp());
        assert_eq!(20, char.current_mp);

        // push out to unequip
        char.unequip_ring(&Key::Ring(Ring::MP));
        assert_eq!(15, char.max_mp());
        assert_eq!(15, char.current_mp);

        char.unequip_ring(&Key::Ring(Ring::MP));
        assert_eq!(10, char.max_mp());
        assert_eq!(10, char.current_mp);

        // preserve taken damage
        char.current_mp -= 3;

        char.equip_ring(Ring::MP);
        assert_eq!(15, char.max_mp());
        assert_eq!(12, char.current_mp);

        char.unequip_ring(&Key::Ring(Ring::MP));
        assert_eq!(10, char.max_mp());
        assert_eq!(7, char.current_mp);
    }

    #[test]
    fn test_attack_ring() {
        let mut char = new_plain_stats_char();
        char.class.mp = None;
        assert_eq!(10, char.physical_attack());

        char.equip_ring(Ring::Attack);
        assert_eq!(15, char.physical_attack());
    }

    #[test]
    fn test_deffense_ring() {
        let mut char = new_plain_stats_char();
        assert_eq!(0, char.deffense());

        char.equip_ring(Ring::Deffense);
        assert_eq!(5, char.deffense());
    }

    #[test]
    fn test_magic_ring() {
        let mut char = new_plain_stats_char();
        assert_eq!(30, char.magic_attack());

        char.equip_ring(Ring::Magic);
        assert_eq!(45, char.magic_attack());
    }

    #[test]
    fn test_speed_ring() {
        let mut char = new_plain_stats_char();
        assert_eq!(10, char.speed());

        char.equip_ring(Ring::Speed);
        assert_eq!(15, char.speed());
    }

    #[test]
    fn test_status_rings() {
        let mut char = new_plain_stats_char();
        let mut another = new_plain_stats_char();
        assert!(char.inflicted_status_effect(&another).is_none());

        char.left_ring = Some(Ring::Fire);
        assert_eq!(
            Some((StatusEffect::Burn, 3)),
            char.inflicted_status_effect(&another)
        );

        char.left_ring = None;
        char.right_ring = Some(Ring::Fire);
        assert_eq!(
            Some((StatusEffect::Burn, 3)),
            char.inflicted_status_effect(&another)
        );

        char.left_ring = Some(Ring::Poison);
        char.right_ring = None;
        assert_eq!(
            Some((StatusEffect::Poison, 3)),
            char.inflicted_status_effect(&another)
        );

        char.left_ring = None;
        char.right_ring = Some(Ring::Poison);
        assert_eq!(
            Some((StatusEffect::Poison, 3)),
            char.inflicted_status_effect(&another)
        );

        another.left_ring = Some(Ring::Protect);
        assert!(char.inflicted_status_effect(&another).is_none());

        another.right_ring = Some(Ring::Protect);
        another.left_ring = None;
        assert!(char.inflicted_status_effect(&another).is_none());
    }

    #[test]
    fn modify_stat() {
        let mut char = new_plain_stats_char();
        assert_eq!(10, char.modify_stat(10, Ring::HP));
        char.left_ring = Some(Ring::Void);
        char.right_ring = Some(Ring::Void);
        assert_eq!(10, char.modify_stat(10, Ring::HP));

        char.left_ring = Some(Ring::HP);
        assert_eq!(15, char.modify_stat(10, Ring::HP));

        char.right_ring = Some(Ring::HP);
        assert_eq!(20, char.modify_stat(10, Ring::HP));
    }

    #[test]
    fn magic_attacks() {
        let mut player = Character::player();
        let enemy_base = class::Class::random(class::Category::Common);
        let enemy_class = class::Class {
            speed: class::Stat(1, 1),
            hp: class::Stat(100, 1),
            strength: class::Stat(5, 1),
            ..enemy_base.clone()
        };
        let mut enemy = Character::new(enemy_class, 1);

        player.change_class("mage").unwrap_or_default();
        let player_class = class::Class {
            speed: class::Stat(2, 1),
            hp: class::Stat(20, 1),
            strength: class::Stat(10, 1), // each hit will take 10hp
            mp: Some(class::Stat(10, 1)),
            ..player.class.clone()
        };
        player = Character::new(player_class, 1);

        // mage -mp with enough mp
        player.attack(&mut enemy).1.unwrap();
        assert_eq!(7, player.current_mp);
        assert_eq!(70, enemy.current_hp);

        player.attack(&mut enemy).1.unwrap();
        player.attack(&mut enemy).1.unwrap();
        assert_eq!(1, player.current_mp);
        assert_eq!(10, enemy.current_hp);

        // mage -mp=0 without enough mp
        player.attack(&mut enemy).1.unwrap();
        assert_eq!(1, player.current_mp);
        assert_eq!(7, enemy.current_hp);
    }

    #[test]
    fn test_counter() {
        let mut player = new_char();
        let mut enemy = new_char();

        assert_eq!(25, player.max_hp());
        assert_eq!(25, player.current_hp);

        // basic attack
        let _ = player.attack(&mut enemy);
        assert_eq!(15, enemy.current_hp);

        // shouldn't counter if no ring equipped
        enemy.current_hp = 25;
        player.maybe_counter_attack(&mut enemy);
        assert_eq!(25, enemy.current_hp);

        // counter when ring equipped
        player.left_ring = Some(Ring::Counter);
        player.maybe_counter_attack(&mut enemy);
        assert_eq!(15, enemy.current_hp);

        player.right_ring = Some(Ring::Counter);
        player.left_ring = None;
        enemy.current_hp = 25;
        player.maybe_counter_attack(&mut enemy);
        assert_eq!(15, enemy.current_hp);
    }

    #[test]
    fn test_double_beat() {
        let mut player = new_char();
        let mut enemy = new_char();

        // shouldn't counter if no ring equipped
        enemy.current_hp = 25;
        player.maybe_double_beat(&mut enemy);
        assert_eq!(25, enemy.current_hp);

        // counter when ring equipped
        player.left_ring = Some(Ring::Double);
        player.maybe_double_beat(&mut enemy);
        assert_eq!(15, enemy.current_hp);

        player.right_ring = Some(Ring::Double);
        player.left_ring = None;
        enemy.current_hp = 25;
        player.maybe_double_beat(&mut enemy);
        assert_eq!(15, enemy.current_hp);
    }

    #[test]
    fn test_revive() {
        let mut player = new_char();
        let mut enemy = new_char();

        // no ring -- alive = alive
        let (_, result) = enemy.attack(&mut player);
        assert!(result.is_ok());
        let result = player.maybe_revive(result, false);
        assert!(result.is_ok());

        let (_, result) = enemy.attack(&mut player);
        let result = player.maybe_revive(result, true);
        assert!(result.is_ok());

        // no ring -- dead = dead
        player.current_hp = 5;
        let (_, result) = enemy.attack(&mut player);
        assert!(result.is_err());
        let result = player.maybe_revive(result, false);
        assert!(result.is_err());

        player.current_hp = 5;
        let (_, result) = enemy.attack(&mut player);
        assert!(result.is_err());
        let result = player.maybe_revive(result, true);
        assert!(result.is_err());

        // ring alive = alive
        player.current_hp = 25;
        player.left_ring = Some(Ring::Revive);
        let (_, result) = enemy.attack(&mut player);
        let result = player.maybe_revive(result, false);
        assert!(result.is_ok());

        let (_, result) = enemy.attack(&mut player);
        let result = player.maybe_revive(result, true);
        assert!(result.is_ok());

        // ring dead once = alive
        player.current_hp = 5;
        let (_, result) = enemy.attack(&mut player);
        assert!(result.is_err());
        let result = player.maybe_revive(result, false);
        assert!(result.is_ok());

        // ring dead twice = dead
        assert_eq!(2, player.current_hp);
        let (_, result) = enemy.attack(&mut player);
        let result = player.maybe_revive(result, true);
        assert!(result.is_err());
    }

    #[test]
    fn gold_gained() {
        let mut player = new_char();

        assert_eq!(50, player.gold_gained(1));
        assert_eq!(50, player.gold_gained(2));
        assert_eq!(100, player.gold_gained(3));
        assert_eq!(150, player.gold_gained(4));

        player.left_ring = Some(Ring::Gold);
        assert_eq!(100, player.gold_gained(1));
        assert_eq!(100, player.gold_gained(2));
        assert_eq!(200, player.gold_gained(3));
        assert_eq!(300, player.gold_gained(4));

        player.left_ring = None;
        player.right_ring = Some(Ring::Gold);
        assert_eq!(100, player.gold_gained(1));
        assert_eq!(100, player.gold_gained(2));
        assert_eq!(200, player.gold_gained(3));
        assert_eq!(300, player.gold_gained(4));
    }

    // HELPERS

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

    fn new_plain_stats_char() -> Character {
        Character {
            max_hp: 10,
            current_hp: 10,
            max_mp: 10,
            current_mp: 10,
            strength: 10,
            speed: 10,
            class: Class::player_by_name("mage").unwrap().clone(),
            ..Character::default()
        }
    }
}
