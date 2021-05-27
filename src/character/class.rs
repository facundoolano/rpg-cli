use crate::location;
use rand::prelude::SliceRandom;

#[derive(Debug)]
struct Stat {
    pub base: i32,
    // TODO does it make sense to make this a rate, and make those calculations, if it's always going to be fixed?
    pub inc_rate: f64,
}

impl Stat {
    pub fn at(&self, level: i32) -> i32 {
        self.base + level * self.increase()
    }

    pub fn increase(&self) -> i32 {
        (self.base as f64 * self.inc_rate).ceil() as i32
    }
}

/// Character classes, which will determine the parameters to start and
/// increase the stats of the character.
#[derive(Debug)]
pub struct Class {
    pub name: &'static str,

    pub hp: Stat,
    pub strength: Stat,
    pub speed: Stat,
}

impl Class {
    pub const HERO: Self = Self {
        name: "hero",
        hp: Stat {
            base: 25,
            inc_rate: 0.25,
        },
        strength: Stat {
            base: 10,
            inc_rate: 0.2,
        },
        speed: Stat {
            base: 5,
            inc_rate: 0.1,
        },
    };

    pub fn random_enemy(distance: location::Distance) -> &'static Self {
        weighted_choice(distance)
    }
}

// Enemy classes are grouped into near/mid/far groups
const NEAR_ENEMIES: &[Class] = &[RAT, WOLF, SNAKE, SLIME, SPIDER];
const MEDIUM_ENEMIES: &[Class] = &[ZOMBIE, ORC, SKELETON, DEMON, VAMPIRE, DRAGON, GOLEM];
const FAR_ENEMIES: &[Class] = &[CHIMERA, BASILISK, MINOTAUR, BALROG, PHOENIX];

/// Choose an enemy randomly, with higher chance to difficult enemies the further from home.
fn weighted_choice(distance: location::Distance) -> &'static Class {
    // the weights for each group of enemies are different depending on the distance
    // the further from home, the bigger the chance to find difficult enemies
    let (w_near, w_mid, w_far) = match distance {
        location::Distance::Near(_) => (9, 2, 0),
        location::Distance::Mid(_) => (7, 10, 1),
        location::Distance::Far(_) => (1, 6, 3),
    };

    // assign weights to each group
    let near = NEAR_ENEMIES.iter().map(|c| (c, w_near));
    let mid = MEDIUM_ENEMIES.iter().map(|c| (c, w_mid));
    let far = FAR_ENEMIES.iter().map(|c| (c, w_far));

    // make a weighted random choice
    let mut rng = rand::thread_rng();
    near.chain(mid)
        .chain(far)
        .collect::<Vec<(&Class, i32)>>()
        .as_slice()
        .choose_weighted(&mut rng, |(_c, weight)| *weight)
        .unwrap()
        .0
}

// NOTE: we shouldn't end up in a place were the hero raises its value and as
// a consequence the enemies raise it too, making them unbeatable.
// Consider: 1. raising the enemy level solely (or primarily) based on distance;
// 2. decreasing rates to prevent overgrowth at higher levels
// as a starting measure, using increase rates way below those of the player

/// Defaults for all enemies.
/// For when it's not obvious how a given class would differ from the resst.
const BASE: Class = Class {
    name: "enemy",
    hp: Stat {
        base: 20,
        inc_rate: 0.10,
    },
    strength: Stat {
        base: 10,
        inc_rate: 0.07,
    },
    speed: Stat {
        base: 3,
        inc_rate: 0.07,
    },
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
    start_strength: 45,
    start_speed: 1,

    speed_rate: 0.04,

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
