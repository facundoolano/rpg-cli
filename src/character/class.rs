use crate::location;
use rand::prelude::SliceRandom;

#[derive(Debug)]
pub struct Stat(pub i32, pub i32);

/// FIXME
impl Stat {
    pub fn base(&self) -> i32 {
        self.0
    }

    pub fn increase(&self) -> i32 {
        self.1
    }

    pub fn at(&self, level: i32) -> i32 {
        self.0 + level * self.increase()
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
        hp: Stat(25, 7),
        strength: Stat(10, 3),
        speed: Stat(10, 2),
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

const RAT: Class = Class {
    name: "rat",
    hp: Stat(10, 3),
    strength: Stat(5, 2),
    speed: Stat(16, 2),
};

const WOLF: Class = Class {
    name: "wolf",
    hp: Stat(15, 3),
    strength: Stat(8, 2),
    speed: Stat(12, 2),
};

const SNAKE: Class = Class {
    name: "snake",
    hp: Stat(13, 3),
    strength: Stat(7, 2),
    speed: Stat(6, 2),
};

const SLIME: Class = Class {
    name: "slime",
    hp: Stat(80, 3),
    strength: Stat(3, 2),
    speed: Stat(4, 2),
};

const SPIDER: Class = Class {
    name: "spider",
    hp: Stat(10, 3),
    strength: Stat(9, 2),
    speed: Stat(12, 2),
};

const ZOMBIE: Class = Class {
    name: "zombie",
    hp: Stat(50, 3),
    strength: Stat(8, 2),
    speed: Stat(6, 2),
};

const ORC: Class = Class {
    name: "orc",
    hp: Stat(35, 3),
    strength: Stat(13, 2),
    speed: Stat(12, 2),
};

const SKELETON: Class = Class {
    name: "skeleton",
    hp: Stat(30, 3),
    strength: Stat(10, 2),
    speed: Stat(10, 2),
};

const DEMON: Class = Class {
    name: "demon",
    hp: Stat(50, 3),
    strength: Stat(10, 2),
    speed: Stat(18, 2),
};

const VAMPIRE: Class = Class {
    name: "vampire",
    hp: Stat(50, 3),
    strength: Stat(13, 2),
    speed: Stat(10, 2),
};

const DRAGON: Class = Class {
    name: "dragon",
    hp: Stat(100, 3),
    strength: Stat(25, 2),
    speed: Stat(8, 2),
};

const GOLEM: Class = Class {
    name: "golem",
    hp: Stat(50, 3),
    strength: Stat(45, 2),
    speed: Stat(2, 1),
};

const CHIMERA: Class = Class {
    name: "chimera",
    hp: Stat(200, 2),
    strength: Stat(90, 2),
    speed: Stat(16, 2),
};

const BASILISK: Class = Class {
    name: "basilisk",
    hp: Stat(150, 3),
    strength: Stat(100, 2),
    speed: Stat(18, 2),
};

const MINOTAUR: Class = Class {
    name: "minotaur",
    hp: Stat(100, 3),
    strength: Stat(60, 2),
    speed: Stat(40, 2),
};

const BALROG: Class = Class {
    name: "balrog",
    hp: Stat(200, 3),
    strength: Stat(200, 2),
    speed: Stat(14, 2),
};

const PHOENIX: Class = Class {
    name: "phoenix",
    hp: Stat(350, 3),
    strength: Stat(180, 2),
    speed: Stat(28, 2),
};
