use super::Item;
use crate::game;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Rings are a wearable item that produce arbitrary effects hooked in
/// different places of the game, e.g. increase a stat, double gold gained, etc.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, EnumIter, Debug)]
pub enum Ring {
    /// No-effect ring
    Void,

    /// Increases physical attack
    Attack,

    /// Increases deffense
    Deffense,

    /// Increases speed stat
    Speed,

    /// Increases magical attack
    Magic,

    /// Increases max MP
    MP,

    /// Increases max HP
    HP,

    /// Enemies don't appear while wearing this ring
    Evade,
}

impl Ring {
    pub fn set() -> HashSet<Ring> {
        Ring::iter().collect()
    }

    // FIXME consider this key to be a standard item thing
    pub fn key(&self) -> &'static str {
        match self {
            Ring::Void => "void-rng",
            Ring::Attack => "att-rng",
            Ring::Deffense => "def-rng",
            Ring::Speed => "spd-rng",
            Ring::Magic => "mag-rng",
            Ring::MP => "mp-rng",
            Ring::HP => "hp-rng",
            Ring::Evade => "evade-rng",
        }
    }

    /// For stat modifying stats, return the factor that should be
    /// applied to the base character stat.
    pub fn factor(&self) -> f64 {
        match self {
            Ring::Attack => 0.5,
            Ring::Deffense => 0.5,
            Ring::Speed => 0.5,
            Ring::Magic => 0.5,
            Ring::MP => 0.5,
            Ring::HP => 0.5,
            _ => 0.0,
        }
    }
}

impl fmt::Display for Ring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key())
    }
}

#[typetag::serde]
impl Item for Ring {
    /// When the ring is used, equip in the player. If the player was already
    /// wearing two rings, move the second one back to the inventory.
    fn apply(&mut self, game: &mut game::Game) {
        if let Some(removed) = game.player.equip_ring(self.clone()) {
            game.add_item(&removed.to_string(), Box::new(removed));
        }
    }
}
