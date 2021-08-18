use core::fmt;

use crate::character::class as character;
use serde::{Deserialize, Serialize};

/// TODO document
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Equipment {
    pub sword: Option<Weapon>,
    pub shield: Option<Weapon>,
}

impl Equipment {
    pub fn new() -> Self {
        Self {
            sword: None,
            shield: None,
        }
    }

    pub fn attack(&self) -> i32 {
        self.sword.as_ref().map_or(0, |s| s.strength())
    }

    pub fn deffense(&self) -> i32 {
        self.shield.as_ref().map_or(0, |s| s.strength())
    }
}

// TODO move to separate modules

/// TODO document
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Weapon {
    pub level: i32,
    name: String,
}

impl Weapon {
    pub fn sword(level: i32) -> Self {
        Self {
            name: "sword".to_string(),
            level,
        }
    }

    pub fn shield(level: i32) -> Self {
        Self {
            name: "shield".to_string(),
            level,
        }
    }

    /// How many strength points get added to the player when
    /// the item is equipped.
    pub fn strength(&self) -> i32 {
        // get the base strength of the hero at this level
        let player_strength = character::Class::player_first().strength.at(self.level);

        // calculate the added strength as a function of the player strength
        (player_strength as f64 * 0.5).round() as i32
    }

    pub fn is_upgrade_from(&self, maybe_other: &Option<Self>) -> bool {
        if let Some(equip) = maybe_other {
            self.level > equip.level
        } else {
            true
        }
    }
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.name, self.level)
    }
}
