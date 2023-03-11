use super::{key, Item};
use crate::game;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Rings are a wearable item that produce arbitrary effects hooked in
/// different places of the game, e.g. increase a stat, double gold gained, etc.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, EnumIter, Debug, Default)]
pub enum Ring {
    #[default]
    Void,
    Attack,
    Deffense,
    Speed,
    Magic,
    MP,
    HP,
    Evade,
    RegenHP,
    RegenMP,
    Ruling,
    Protect,
    Fire,
    Poison,
    Double,
    Counter,
    Revive,
    Chest,
    Gold,
    Diamond,
}

impl Ring {
    pub fn set() -> HashSet<Ring> {
        Ring::iter().collect()
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
            game.add_item(Box::new(removed));
        }
    }

    fn key(&self) -> key::Key {
        key::Key::Ring(self.clone())
    }

    fn describe(&self) -> String {
        let str = match self {
            Ring::Void => "no-effect ring",
            Ring::Attack => "increases physical attack",
            Ring::Deffense => "increases defense",
            Ring::Speed => "increases speed",
            Ring::Magic => "increases magical attack",
            Ring::MP => "increases max mp",
            Ring::HP => "increases max hp",
            Ring::Evade => "reduces enemy appearance frequency",
            Ring::RegenHP => "recovers hp on every turn",
            Ring::RegenMP => "recovers mp on every turn",
            Ring::Ruling => "one ring to rule them all",
            Ring::Protect => "prevents status ailments",
            Ring::Fire => "inflicts burn status on attack",
            Ring::Poison => "inflicts poison status on attack",
            Ring::Double => "strike twice per turn",
            Ring::Counter => "counter-attack when an attack is received",
            Ring::Revive => "come back from dead during battle",
            Ring::Chest => "doubles chest finding frequency",
            Ring::Gold => "doubles gold gained in battles and chests",
            Ring::Diamond => "looks expensive",
        };
        str.to_string()
    }
}
