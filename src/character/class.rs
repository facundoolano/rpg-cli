use serde::{Deserialize, Serialize};

/// Character classes, which will determine the parameters to start and
/// increase the stats of the character. For now generic hero/enemy but
/// should enable multiple player and enemy types.
#[derive(Serialize, Deserialize, Debug)]
pub enum Class {
    Hero,
    Enemy,
    Test,
}

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
    pub fn params_for(class: &Class) -> Parameters {
        match class {
            Class::Hero => HERO_PARAMS,
            Class::Enemy => ENEMY_PARAMS,
            Class::Test => TEST_PARAMS,
        }
    }
}
