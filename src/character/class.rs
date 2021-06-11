use crate::location;
use once_cell::sync::OnceCell;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};

/// A stat represents an attribute of a character, such as strength or speed.
/// This struct contains a stat starting value and the amount that should be
/// applied when the level increases.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Stat {
    pub base: i32,
    pub increase: i32,
}

impl Stat {
    pub fn new(base: i32, increase: i32) -> Self {
        Self { base, increase }
    }

    pub fn base(&self) -> i32 {
        self.base
    }

    pub fn increase(&self) -> i32 {
        self.increase
    }

    pub fn at(&self, level: i32) -> i32 {
        self.base() + level * self.increase()
    }
}

/// Classes are archetypes for characters.
/// The struct contains a specific stat configuration such that all instances of
/// the class have a similar combat behavior.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Class {
    pub name: String,

    pub hp: Stat,
    pub strength: Stat,
    pub speed: Stat,
    pub distance: Option<location::Distance>,
}

impl Class {
    pub fn default_hero() -> Self {
        Self {
            name: "hero".to_string(),
            hp: Stat::new(30, 7),
            strength: Stat::new(12, 3),
            speed: Stat::new(11, 2),
            distance: None,
        }
    }

    pub fn random_enemy(distance: location::Distance) -> Self {
        weighted_choice(distance)
    }
}

// Enemy classes are grouped into near/mid/far groups
pub static NEAR_ENEMIES: OnceCell<Vec<Class>> = OnceCell::new();
pub static MEDIUM_ENEMIES: OnceCell<Vec<Class>> = OnceCell::new();
pub static FAR_ENEMIES: OnceCell<Vec<Class>> = OnceCell::new();

/// Choose an enemy randomly, with higher chance to difficult enemies the further from home.
fn weighted_choice(distance: location::Distance) -> Class {
    // the weights for each group of enemies are different depending on the distance
    // the further from home, the bigger the chance to find difficult enemies
    let (w_near, w_mid, w_far) = match distance {
        location::Distance::Near(_) => (9, 2, 0),
        location::Distance::Mid(_) => (7, 10, 1),
        location::Distance::Far(_) => (1, 6, 3),
    };

    // assign weights to each group
    let near = NEAR_ENEMIES
        .get_or_init(Vec::new)
        .iter()
        .map(|c| (c, w_near));
    let mid = MEDIUM_ENEMIES
        .get_or_init(Vec::new)
        .iter()
        .map(|c| (c, w_mid));
    let far = FAR_ENEMIES
        .get_or_init(Vec::new)
        .iter()
        .map(|c| (c, w_far));

    // make a weighted random choice
    let mut rng = rand::thread_rng();
    near.chain(mid)
        .chain(far)
        .collect::<Vec<(&Class, i32)>>()
        .as_slice()
        .choose_weighted(&mut rng, |(_c, weight)| *weight)
        .unwrap()
        .0
        .clone()
}

// NOTE: we shouldn't end up in a place were the hero raises its value and as
// a consequence the enemies raise it too, making them unbeatable.
// Consider: 1. raising the enemy level solely (or primarily) based on distance;
// 2. decreasing rates to prevent overgrowth at higher levels
// as a starting measure, using increase rates way below those of the player
