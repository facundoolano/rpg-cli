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

pub fn customize(bytes: &[u8]) {
    CLASSES.set(from_bytes(bytes)).unwrap();
}

fn default_classes() -> HashMap<String, Vec<Class>> {
    from_bytes(include_bytes!("classes.yaml"))
}

fn from_bytes(bytes: &[u8]) -> HashMap<String, Vec<Class>> {
    // it would arguably be better for these module not to deal with deserialization
    // and yaml, but at this stage it's easier to assume the defaults when customize
    // is not called, especially for tests.
    let mut classes: Vec<Class> = serde_yaml::from_slice(bytes).unwrap();

    let mut class_groups = HashMap::new();
    for class in classes.drain(..) {
        let entry = class_groups
            .entry(class.group.clone())
            .or_insert_with(Vec::new);
        entry.push(class);
    }
    class_groups
}

impl Class {
    // TODO consider making all module level or all struct level
    pub fn warrior() -> &'static Self {
        CLASSES
            .get_or_init(default_classes)
            .get("player")
            .unwrap()
            .get(0)
            .unwrap()
    }

    pub fn random_enemy(distance: location::Distance) -> Self {
        weighted_choice(distance)
    }

    pub fn enemy_names(group: &str) -> HashSet<String> {
        CLASSES
            .get_or_init(default_classes)
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
