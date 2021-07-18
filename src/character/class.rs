use crate::location;
use once_cell::sync::OnceCell;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A stat represents an attribute of a character, such as strength or speed.
/// This struct contains a stat starting value and the amount that should be
/// applied when the level increases.
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

    pub category: Category,

    pub inflicts: Option<(super::StatusEffect, u32)>,
}

/// Determines whether the class is intended for a Player or, if it's for an enemy,
/// How rare it is (how frequently it should appear).
/// Enables easier customization of the classes via an external file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Player,
    Common,
    Rare,
    Legendary,
}

static CLASSES: OnceCell<HashMap<Category, Vec<Class>>> = OnceCell::new();

impl Class {
    /// Customize the classes definitions based on an input yaml byte array.
    pub fn load(bytes: &[u8]) {
        CLASSES.set(from_bytes(bytes)).unwrap();
    }

    /// The default player class, exposed for initialization and parameterization of
    /// items and equipment.
    pub fn player_default() -> &'static Self {
        CLASSES
            .get_or_init(default_classes)
            .get(&Category::Player)
            .unwrap()
            .get(0)
            .unwrap()
    }

    pub fn random_enemy(distance: location::Distance) -> Self {
        weighted_choice(distance)
    }

    pub fn enemy_names(group: Category) -> HashSet<String> {
        CLASSES
            .get_or_init(default_classes)
            .get(&group)
            .unwrap()
            .iter()
            .map(|class| class.name.clone())
            .collect()
    }
}

fn default_classes() -> HashMap<Category, Vec<Class>> {
    from_bytes(include_bytes!("classes.yaml"))
}

fn from_bytes(bytes: &[u8]) -> HashMap<Category, Vec<Class>> {
    // it would arguably be better for these module not to deal with deserialization
    // and yaml, but at this stage it's easier allow it to pick up defaults from
    // the local file when it hasn't been customized (especially for tests)
    let mut classes: Vec<Class> = serde_yaml::from_slice(bytes).unwrap();

    let mut class_groups = HashMap::new();
    for class in classes.drain(..) {
        let entry = class_groups
            .entry(class.category.clone())
            .or_insert_with(Vec::new);
        entry.push(class);
    }
    class_groups
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
        (Category::Common, w_common),
        (Category::Rare, w_rare),
        (Category::Legendary, w_legendary),
    ];
    let category = &weights
        .as_slice()
        .choose_weighted(&mut rng, |(_c, weight)| *weight)
        .unwrap()
        .0;

    // get a random class within the group
    let classes = CLASSES.get().unwrap().get(category).unwrap();
    classes.choose(&mut rng).unwrap().clone()
}
