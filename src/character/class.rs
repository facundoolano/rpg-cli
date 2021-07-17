use serde::{Deserialize, Serialize};
use crate::location;
use rand::prelude::SliceRandom;
use once_cell::sync::OnceCell;
use std::collections::HashMap;

/// A stat represents an attribute of a character, such as strength or speed.
/// This struct contains a stat starting value and the amount that should be
/// applied when the level increases.
// TODO check if we still need clone
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stat(pub i32, pub i32);

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

/// Classes are archetypes for characters.
/// The struct contains a specific stat configuration such that all instances of
/// the class have a similar combat behavior.
// TODO check if we still need clone
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Class {
    pub name: String,

    pub hp: Stat,
    pub strength: Stat,
    pub speed: Stat,

    // FIXME probably better to make this an Enum
    group: String,

    pub inflicts: Option<(super::StatusEffect, u32)>,
}

static CLASSES: OnceCell<HashMap<String, Vec<Class>>> = OnceCell::new();

pub fn init() {
    // TODO allow to load from a provided classes file alternatively
    // maybe all of this should be move to the file handling section
    let class_bytes = include_bytes!("classes.yaml");
    let mut classes: Vec<Class> = serde_yaml::from_slice(class_bytes).unwrap();

    let mut class_groups = HashMap::new();
    for class in classes.drain(..) {
        let entry = class_groups.entry(class.group.clone())
            .or_insert_with(Vec::new);
        entry.push(class);
    }
    CLASSES.set(class_groups).unwrap();
}



impl Class {
    // TODO consider making all module level or all struct level
    pub fn hero() -> Self {
        // FIXME it's inelegant to be creating a new one in each call to this
        // especially calls made just to check stats

        // This is famously the worst line of Rust ever
        CLASSES.get().unwrap().get("player").unwrap().get(0).unwrap().clone()
    }

    pub fn random_enemy(distance: location::Distance) -> Self {
        weighted_choice(distance)
    }
}

pub const COMMON: &[Class] = &[];
pub const RARE: &[Class] = &[];
pub const LEGENDARY: &[Class] = &[];

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
    let near = COMMON.iter().map(|c| (c, w_near));
    let mid = RARE.iter().map(|c| (c, w_mid));
    let far = LEGENDARY.iter().map(|c| (c, w_far));

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
