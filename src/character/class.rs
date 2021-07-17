use crate::location;
use once_cell::sync::OnceCell;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
    pub group: String,

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
        let entry = class_groups
            .entry(class.group.clone())
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
        CLASSES
            .get()
            .unwrap()
            .get("player")
            .unwrap()
            .get(0)
            .unwrap()
            .clone()
    }

    pub fn random_enemy(distance: location::Distance) -> Self {
        weighted_choice(distance)
    }

    pub fn enemy_names(group: &str) -> HashSet<String> {
        CLASSES
            .get()
            .unwrap()
            .get(group)
            .unwrap()
            .iter()
            .map(|class| class.name.clone())
            .collect()
    }
}

/// Choose an enemy randomly, with higher chance to difficult enemies the further from home.
fn weighted_choice(distance: location::Distance) -> Class {
    // the weights for each group of enemies are different depending on the distance
    // the further from home, the bigger the chance to find difficult enemies
    let (w_common, w_rare, w_legendary) = match distance {
        location::Distance::Near(_) => (9, 2, 0),
        location::Distance::Mid(_) => (7, 10, 1),
        location::Distance::Far(_) => (1, 6, 3),
    };

    let mut rng = rand::thread_rng();

    // assign weights to each group and select one
    let weights = vec![
        ("common", w_common),
        ("rare", w_rare),
        ("legendary", w_legendary),
    ];
    let group = weights
        .as_slice()
        .choose_weighted(&mut rng, |(_c, weight)| *weight)
        .unwrap()
        .0;

    // get a random class within the group
    let classes = CLASSES.get().unwrap().get(group).unwrap();
    classes.choose(&mut rng).unwrap().clone()
}
