use rand::seq::IteratorRandom;

/// Character classes, which will determine the parameters to start and
/// increase the stats of the character.
#[derive(Debug)]
pub struct Class {
    pub name: &'static str,
    pub start_hp: i32,
    pub start_strength: i32,
    pub start_speed: i32,

    pub hp_rate: f64,
    pub strength_rate: f64,
    pub speed_rate: f64,
}

impl Class {
    pub const HERO: Self = Self {
        name: "hero",
        start_hp: 25,
        start_strength: 10,
        start_speed: 5,

        hp_rate: 0.3,
        strength_rate: 0.2,
        speed_rate: 0.1,
    };

    pub fn strength_at(&self, level: i32) -> i32 {
        let inc_rate = 1.0 + self.strength_rate;
        (self.start_strength as f64 * inc_rate.powi(level)) as i32
    }

    pub fn random_enemy(distance_from_home: i32) -> &'static Self {
        // TODO use weights instead of separate lists
        // e.g. when > 4 distance more likely to get medium
        // but not impossible to get near enemies
        // e.g. with bad luck you could find a boss in medium distance
        match distance_from_home {
            n if n <= 4 => random_choice(NEAR_ENEMIES),
            n if n <= 9 => random_choice(MEDIUM_ENEMIES),
            _ => random_choice(FAR_ENEMIES),
        }
    }
}

// At the moment the only criteria to choose one enemy class vs another is how far
// from home they appear. Within the same group, the class is chosen randomly.
const NEAR_ENEMIES: &[Class] = &[RAT, WOLF, SNAKE, SLIME, SPIDER];
const MEDIUM_ENEMIES: &[Class] = &[ZOMBIE, ORC, SKELETON, DEMON, VAMPIRE, DRAGON, GOLEM];
const FAR_ENEMIES: &[Class] = &[CHIMERA, BASILISK, MINOTAUR, BALROG, PHOENIX];

fn random_choice(options: &[Class]) -> &Class {
    let mut rng = rand::thread_rng();
    options.iter().choose(&mut rng).unwrap()
}

// TODO verify that the rates produce some realistic values for far enemies especially for far enemies
// which will only appear at high levels (i.e. the high start values can grow too big at their actual level)
// we shouldn't end up in a place were the hero raises its value and as a consequence the enemies
// raise it too.
// Consider: 1. raising the enemy level solely (or primarily) based on distance;
// 2. decreasing rates to prevent overgrowth at higher levels
// as a starting measure, using increase rates way below those of the player

/// Defaults for all enemies.
/// For when it's not obvious how a given class would differ from the resst.
const BASE: Class = Class {
    name: "enemy",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.10,
    strength_rate: 0.05,
    speed_rate: 0.05,
};

const RAT: Class = Class {
    name: "rat",
    start_hp: 10,
    start_strength: 5,
    start_speed: 8,

    ..BASE
};

const WOLF: Class = Class {
    name: "wolf",
    start_hp: 15,
    start_strength: 8,
    start_speed: 6,

    ..BASE
};

const SNAKE: Class = Class {
    name: "snake",
    start_hp: 13,
    start_strength: 7,
    start_speed: 3,

    ..BASE
};

const SLIME: Class = Class {
    name: "slime",
    start_hp: 100,
    start_strength: 3,
    start_speed: 2,

    ..BASE
};

const SPIDER: Class = Class {
    name: "spider",
    start_hp: 10,
    start_strength: 9,
    start_speed: 6,

    ..BASE
};

const ZOMBIE: Class = Class {
    name: "zombie",
    start_hp: 50,
    start_strength: 8,
    start_speed: 3,

    ..BASE
};

const ORC: Class = Class {
    name: "orc",
    start_hp: 35,
    start_strength: 13,
    start_speed: 6,

    ..BASE
};

const SKELETON: Class = Class {
    name: "skeleton",
    start_hp: 30,
    start_strength: 10,
    start_speed: 5,

    ..BASE
};

const DEMON: Class = Class {
    name: "demon",
    start_hp: 50,
    start_strength: 10,
    start_speed: 9,

    ..BASE
};

const VAMPIRE: Class = Class {
    name: "vampire",
    start_hp: 50,
    start_strength: 13,
    start_speed: 5,

    ..BASE
};

const DRAGON: Class = Class {
    name: "dragon",
    start_hp: 100,
    start_strength: 25,
    start_speed: 4,

    ..BASE
};

const GOLEM: Class = Class {
    name: "golem",
    start_hp: 50,
    start_strength: 50,
    start_speed: 2,

    ..BASE
};

const CHIMERA: Class = Class {
    name: "chimera",
    start_hp: 200,
    start_strength: 90,
    start_speed: 8,

    ..BASE
};

const BASILISK: Class = Class {
    name: "basilisk",
    start_hp: 150,
    start_strength: 100,
    start_speed: 9,

    ..BASE
};

const MINOTAUR: Class = Class {
    name: "minotaur",
    start_hp: 100,
    start_strength: 60,
    start_speed: 20,

    ..BASE
};

const BALROG: Class = Class {
    name: "balrog",
    start_hp: 200,
    start_strength: 200,
    start_speed: 7,

    ..BASE
};

const PHOENIX: Class = Class {
    name: "phoenix",
    start_hp: 350,
    start_strength: 180,
    start_speed: 14,

    ..BASE
};
