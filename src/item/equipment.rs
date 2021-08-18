use core::fmt;

use crate::character::class as character;
use serde::{Deserialize, Serialize};

/// Packages together different equipment pieces that can be worn by characters
/// or found in chests.
/// Provides a unified interface for stat contributions to the base stats of a
/// characters, e.g. the increased attack contributed by a sword and the
/// deffense contributed by a shield.
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

    /// Take the sword and/or shield from the given equipment if
    /// self has none or has one with lower level.
    /// Returns a tuple indicating whether there were (sword, shield) upgrades.
    pub fn upgrade(&mut self, other: &mut Self) -> (bool, bool) {
        (
            maybe_upgrade(&mut self.sword, &mut other.sword),
            maybe_upgrade(&mut self.shield, &mut other.shield),
        )
    }

    pub fn attack(&self) -> i32 {
        self.sword.as_ref().map_or(0, |s| s.strength())
    }

    pub fn deffense(&self) -> i32 {
        self.shield.as_ref().map_or(0, |s| s.strength())
    }
}

fn maybe_upgrade(current: &mut Option<Weapon>, other: &mut Option<Weapon>) -> bool {
    if let Some(shield) = other.take() {
        if shield.is_upgrade_from(current) {
            current.replace(shield);
            return true;
        }
    }
    false
}

/// Equipment piece with a strength contribution based on
/// a level. Used to generically represent swords and shields.
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

    /// Return true if the other weapon either is None or has lower level than this one.
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
