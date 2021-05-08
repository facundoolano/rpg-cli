use std::fmt;
use rand::seq::IteratorRandom;

use serde::{Deserialize, Serialize};

/// Character classes, which will determine the parameters to start and
/// increase the stats of the character. For now generic hero/enemy but
/// should enable multiple player and enemy types.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Class {
    Hero,
    Test,

    Rat,
    Wolf,
    Snake,
    Slime,
    Spider,

    Zombie,
    Orc,
    Skeleton,
    Demon,
    Vampire,
    Dragon,
    Golem,

    Chimera,
    Basilisk,
    Minotaur,
    Balrog,
    Phoenix,
}

// FIXME there's too much duplication with this enum approach
// explore if using structs or traits could make things simpler somehow

const NEAR_ENEMIES: &[Class] = &[
    Class::Rat,
    Class::Wolf,
    Class::Snake,
    Class::Slime,
    Class::Spider,
];

const MEDIUM_ENEMIES: &[Class] = &[
    Class::Zombie,
    Class::Orc,
    Class::Skeleton,
    Class::Demon,
    Class::Vampire,
    Class::Dragon,
    Class::Golem,
];

const FAR_ENEMIES: &[Class] = &[
    Class::Chimera,
    Class::Basilisk,
    Class::Minotaur,
    Class::Balrog,
    Class::Phoenix,
];

/// The stat configuration for a given character class.
/// It determines the default values for stat and the rate at
/// which they increse on level up.
pub struct Parameters {
    pub start_hp: i32,
    pub start_strength: i32,
    pub start_speed: i32,

    pub hp_rate: f64,
    pub strength_rate: f64,
    pub speed_rate: f64,
}

const HERO_PARAMS: Parameters = Parameters {
    start_hp: 25,
    start_strength: 10,
    start_speed: 5,

    hp_rate: 0.3,
    strength_rate: 0.2,
    speed_rate: 0.1,
};

const ENEMY_PARAMS: Parameters = Parameters {
    start_hp: 20,
    start_strength: 10,
    start_speed: 3,

    hp_rate: 0.20,
    strength_rate: 0.15,
    speed_rate: 0.07,
};

// this class is left fixed to use in unit tests so they don't break
// every time we tune rest of the classes's parameters
const TEST_PARAMS: Parameters = Parameters {
    start_hp: 25,
    start_strength: 10,
    start_speed: 5,

    hp_rate: 0.3,
    strength_rate: 0.1,
    speed_rate: 0.1,
};

impl Class {
    pub fn params(&self) -> Parameters {
        match self {
            Class::Hero => HERO_PARAMS,
            Class::Test => TEST_PARAMS,
            // FIXME make different params per class
            _ => ENEMY_PARAMS,
        }
    }

    pub fn random_enemy(distance_from_home: i32) -> Self {
        match distance_from_home {
            n if n <= 4  => Self::random_choice(NEAR_ENEMIES),
            n if n <= 9  => Self::random_choice(MEDIUM_ENEMIES),
            _ => Self::random_choice(FAR_ENEMIES)
        }
    }

    fn random_choice(options: &[Self]) -> Self {
        let mut rng = rand::thread_rng();
        *options.iter().choose(&mut rng).unwrap()
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // FIXME add padding
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}
