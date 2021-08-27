use core::fmt;

use super::key::Key;
use crate::character::class::Class;
use serde::{Deserialize, Serialize};

/// Equipment piece with a strength contribution based on
/// a level. Used to generically represent swords and shields.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Equipment(Key, i32);

impl Equipment {
    pub fn sword(level: i32) -> Self {
        Self(Key::Sword, level)
    }

    pub fn shield(level: i32) -> Self {
        Self(Key::Shield, level)
    }

    pub fn level(&self) -> i32 {
        self.1
    }

    pub fn key(&self) -> Key {
        self.0.clone()
    }

    /// How many strength points get added to the player when
    /// the item is equipped.
    pub fn strength(&self) -> i32 {
        // get the base strength of the hero at this level
        let player_strength = Class::player_first().strength.at(self.level());

        // calculate the added strength as a function of the player strength
        (player_strength as f64 * 0.5).round() as i32
    }

    /// Return true if the other weapon either is None or has lower level than this one.
    pub fn is_upgrade_from(&self, maybe_other: &Option<Self>) -> bool {
        if let Some(equip) = maybe_other {
            self.level() > equip.level()
        } else {
            true
        }
    }
}

impl fmt::Display for Equipment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.key(), self.level())
    }
}
