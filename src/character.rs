use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::max;

/// Character classes, which will determine the parameters to start and
/// increase the stats of the character. For now generic hero/enemy but
/// should enable multiple player and enemy types.
#[derive(Serialize, Deserialize, Debug)]
enum Class {
    Hero,
    Enemy,
}

struct Parameters {
    start_hp: i32,
    start_strength: i32,
    start_speed: i32,

    hp_rate: f64,
    strength_rate: f64,
    speed_rate: f64,
}

impl Parameters {
    fn of(class: &Class) -> Self {
        match class {
            Class::Hero => Self {
                start_hp: 25,
                start_strength: 10,
                start_speed: 5,

                hp_rate: 0.3,
                strength_rate: 0.1,
                speed_rate: 0.1,
            },
            Class::Enemy => Self {
                start_hp: 20,
                start_strength: 10,
                start_speed: 3,

                hp_rate: 0.20,
                strength_rate: 0.1,
                speed_rate: 0.07,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    class: Class,
    pub name: String,

    pub level: i32,
    pub xp: i32,

    pub max_hp: i32,
    pub current_hp: i32,

    pub strength: i32,
    pub speed: i32,
}

impl Character {
    pub fn player() -> Self {
        Self::new(Class::Hero, "hero", 1)
    }

    pub fn enemy(level: i32) -> Self {
        Self::new(Class::Enemy, "enemy", level)
    }

    fn new(class: Class, name: &str, level: i32) -> Self {
        let params = Parameters::of(&class);
        let mut character = Self {
            class,
            level,
            name: String::from(name),
            xp: 0,
            max_hp: params.start_hp,
            current_hp: params.start_hp,
            strength: params.start_strength,
            speed: params.start_speed,
        };

        for _ in 1..level {
            character.increase_level();
        }

        character
    }

    /// Raise the level and all the character stats.
    fn increase_level(&mut self) {
        let params = Parameters::of(&self.class);

        self.level += 1;
        self.strength = inc(self.strength, params.strength_rate);
        self.speed = inc(self.speed, params.speed_rate);

        // the current should increase proportionally but not
        // erase previous damage
        let previous_damage = self.max_hp - self.current_hp;
        self.max_hp = inc(self.max_hp, params.hp_rate);
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
        // Possible improvements: use different attack and defense stats,
        // incorporate weapon and armor effect.

        let str_10 = self.strength as f64 * 0.1;

        // attenuate the level based difference to help the weaker player
        let level_diff_effect = if self.level < receiver.level {
            (self.level - receiver.level) as f64 * str_10
        } else {
            (self.level - receiver.level) as f64 / 2.0 * str_10
        };

        let damage = self.strength as f64 + level_diff_effect;
        max(str_10.ceil() as i32, randomized(damage) as i32)
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

/// Increase a stat by the given rate, with some randomization.
fn inc(current: i32, rate: f64) -> i32 {
    // if rate is .3, increase can be in .15-.45
    let current_f = current as f64;
    let min = std::cmp::max(1, (current_f * (rate - rate / 2.0)).round() as i32);
    let max = std::cmp::max(1, (current_f * rate + rate / 2.0).round() as i32);

    let mut rng = rand::thread_rng();
    current + rng.gen_range(min..=max)
}

// TODO need a more generic, mockable way of adding variance to stats, damages and other numbers
/// add +/- 20% variance to a f64
/// may make more generic in the future
fn randomized(value: f64) -> i32 {
    let mut rng = rand::thread_rng();
    let min = (value - value * 0.2).floor() as i32;
    let max = (value + value * 0.2).ceil() as i32;
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

        let params = Parameters::of(&Class::Hero);
        assert_eq!(params.start_hp, hero.current_hp);
        assert_eq!(params.start_hp, hero.max_hp);
        assert_eq!(params.start_strength, hero.strength);
        assert_eq!(params.start_speed, hero.speed);
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

        // small numbers
        let value = inc(3, 0.07);
        assert_eq!(4, value);
    }

    #[test]
    fn test_increase_level() {
        let mut hero = Character::player();

        // Using hardcoded start/rates so we can assert with specific values
        // TODO add specific test character class that we can assume won't change
        let params = Parameters::of(&Class::Hero);
        assert_eq!(0.3, params.hp_rate);
        assert_eq!(0.1, params.strength_rate);
        assert_eq!(0.1, params.speed_rate);
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

    #[test]
    fn test_damage() {
        let mut hero = Character::player();
        let mut foe = Character::enemy(1);

        // 1 vs 1 -- no level-based effect
        hero.strength = 10;
        foe.strength = 10;

        let damage = hero.damage(&foe);
        assert!((8..=12).contains(&damage), "value was {}", damage);

        // level 1 vs level 2
        foe.level = 2;
        foe.strength = 15;
        let damage = hero.damage(&foe);
        assert!((7..=11).contains(&damage), "value was {}", damage);

        // level 2 vs level 1
        let damage = foe.damage(&hero);
        assert!((12..=19).contains(&damage), "value was {}", damage);

        // level 1 vs level 5
        foe.level = 5;
        foe.strength = 40;

        let damage = hero.damage(&foe);
        assert!((4..=8).contains(&damage), "value was {}", damage);

        // level 5 vs level 1
        let damage = foe.damage(&hero);
        assert!((38..=58).contains(&damage), "value was {}", damage);
    }

    #[test]
    fn test_xp_gained() {
        let hero = Character::player();
        let mut foe = Character::enemy(1);
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
        let mut hero = Character::player();
        assert_eq!(30, hero.xp_for_next());
        hero.increase_level();
        assert_eq!(84, hero.xp_for_next());
        hero.increase_level();
        assert_eq!(155, hero.xp_for_next());
    }

    #[test]
    fn test_add_experience() {
        let mut hero = Character::player();
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
