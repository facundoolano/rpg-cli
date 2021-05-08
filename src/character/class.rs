use rand::seq::IteratorRandom;

/// Character classes, which will determine the parameters to start and
/// increase the stats of the character.
#[derive(Debug, Clone)]
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

    // At the moment the only criteria to choose one enemy class vs another is how far
    // from home they appear. Within the same group, the classes is chosen randomly.
    const NEAR_ENEMIES: &'static [Self] = &[RAT, WOLF, SNAKE, SLIME, SPIDER];
    const MEDIUM_ENEMIES: &'static [Self] = &[ZOMBIE, ORC, SKELETON, DEMON, VAMPIRE, DRAGON, GOLEM];
    const FAR_ENEMIES: &'static [Self] = &[CHIMERA, BASILISK, MINOTAUR, BALROG, PHOENIX];

    pub fn random_enemy(distance_from_home: i32) -> &'static Self {
        match distance_from_home {
            n if n <= 4 => Self::random_choice(Self::NEAR_ENEMIES),
            n if n <= 9 => Self::random_choice(Self::MEDIUM_ENEMIES),
            _ => Self::random_choice(Self::FAR_ENEMIES),
        }
    }

    fn random_choice(options: &[Self]) -> &Self {
        let mut rng = rand::thread_rng();
        options.iter().choose(&mut rng).unwrap()
    }
}

// TODO review stats
const RAT: Class = Class {
    name: "rat",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const WOLF: Class = Class {
    name: "wolf",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const SNAKE: Class = Class {
    name: "snake",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const SLIME: Class = Class {
    name: "slime",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const SPIDER: Class = Class {
    name: "spider",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const ZOMBIE: Class = Class {
    name: "zombie",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const ORC: Class = Class {
    name: "orc",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const SKELETON: Class = Class {
    name: "skeleton",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const DEMON: Class = Class {
    name: "demon",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const VAMPIRE: Class = Class {
    name: "vampire",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const DRAGON: Class = Class {
    name: "dragon",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const GOLEM: Class = Class {
    name: "golem",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const CHIMERA: Class = Class {
    name: "chimera",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const BASILISK: Class = Class {
    name: "basilisk",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const MINOTAUR: Class = Class {
    name: "minotaur",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const BALROG: Class = Class {
    name: "balrog",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// TODO review stats
const PHOENIX: Class = Class {
    name: "phoenix",
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};
