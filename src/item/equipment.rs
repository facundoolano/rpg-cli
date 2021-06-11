use core::fmt;

use crate::character::class as character;
use serde::{Deserialize, Serialize};

pub trait Equipment: fmt::Display {
    fn new(level: i32) -> Self;

    fn level(&self) -> i32;

    /// How many strength points get added to the player when
    /// the item is equipped.
    fn strength(&self) -> i32 {
        // get the base strength of the hero at this level
        let player_strength = character::Class::default_hero().strength.at(self.level());

        // calculate the added strength as a function of the player strength
        (player_strength as f64 * 0.5).round() as i32
    }

    fn is_upgrade_from(&self, maybe_other: &Option<&Self>) -> bool {
        if let Some(equip) = maybe_other {
            self.level() > equip.level()
        } else {
            true
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sword {
    level: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Shield {
    level: i32,
}

impl fmt::Display for Sword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sword[{}]", self.level())
    }
}

impl Equipment for Sword {
    fn new(level: i32) -> Self {
        Self { level }
    }

    fn level(&self) -> i32 {
        self.level
    }
}

impl fmt::Display for Shield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shield[{}]", self.level())
    }
}

impl Equipment for Shield {
    fn new(level: i32) -> Self {
        Self { level }
    }

    fn level(&self) -> i32 {
        self.level
    }
}
