use super::Item;
use crate::character;
use crate::game;
use serde::{Deserialize, Serialize};

/// Rings are a kind of equipment that produce arbitrary effects hooked in
/// different places of the game, e.g. increase a stat, double gold gained, etc.
#[derive(Serialize, Deserialize, PartialEq)]
pub enum Ring {
    Void,
    Attack,
    Deffense,
    Speed,
    Magic,
    MP,
    HP,
}

impl Ring {
    // TODO should this be to_string instead?
    fn key(&self) -> &'static str {
        match self {
            Ring::Void => "void",
            Ring::Attack => "attack",
            Ring::Deffense => "deffense",
            Ring::Speed => "speed",
            Ring::Magic => "magic",
            Ring::MP => "mp",
            Ring::HP => "hp",
        }
    }

    /// TODO
    fn factor(&self) -> f64 {
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

    /// TODO explain
    fn equip_side_effect(&self, character: &mut character::Character) {
        match self {
            Ring::HP => {
                character.current_hp += (self.factor() * character.max_hp() as f64) as i32;
            }
            Ring::MP => {
                character.current_mp += (self.factor() * character.max_mp() as f64) as i32;
            }
            _ => {}
        }
    }

    /// TODO explain
    fn unequip_side_effect(&self, character: &mut character::Character) {
        match self {
            Ring::HP => {
                let to_remove = (self.factor() * character.max_hp() as f64) as i32;
                character.current_hp += std::cmp::max(1, character.current_hp - to_remove);
            }
            Ring::MP => {
                let to_remove = (self.factor() * character.max_mp() as f64) as i32;
                character.current_mp += std::cmp::max(1, character.current_mp - to_remove);
            }
            _ => {}
        }
    }
}

/// The character is allowed to hold two rings.
/// The ring pair struct is used to hold the rings that the character is wearing,
/// handling the equipping and calculating the net combined effect of the two rings.
#[derive(Serialize, Deserialize, Default)]
pub struct RingPair {
    left: Option<Ring>,
    right: Option<Ring>,
}

// TODO consider removing/reducing this one
// rename to RingSet? RingHolder? RingEquip
impl RingPair {
    pub fn new() -> Self {
        Self {
            left: None,
            right: None,
        }
    }

    /// Put the given ring in the left, moving the left (if any) to the right
    /// and returning the right (if any)
    fn equip(&mut self, ring: Ring) -> Option<Ring> {
        if let Some(old_left) = self.left.replace(ring) {
            return self.right.replace(old_left);
        }
        None
    }

    pub fn attack(&self, strength: i32) -> i32 {
        self.apply(strength, Ring::Attack)
    }

    pub fn deffense(&self, strength: i32) -> i32 {
        self.apply(strength, Ring::Deffense)
    }

    pub fn speed(&self, strength: i32) -> i32 {
        self.apply(strength, Ring::Deffense)
    }

    pub fn magic(&self, base: i32) -> i32 {
        self.apply(base, Ring::Magic)
    }

    pub fn mp(&self, base: i32) -> i32 {
        self.apply(base, Ring::MP)
    }

    pub fn hp(&self, base: i32) -> i32 {
        self.apply(base, Ring::HP)
    }

    /// TODO
    fn apply(&self, base: i32, ring: Ring) -> i32 {
        let factor =
            |r: &Option<Ring>| r.as_ref().filter(|&l| *l == ring).map_or(0.0, Ring::factor);
        let factor = factor(&self.left) + factor(&self.left);
        (base as f64 * factor).round() as i32
    }
}

/// RingItem is a wrapper that lets the rings be added to the inventory and
/// used them to equip them.
#[derive(Serialize, Deserialize)]
pub struct RingItem {
    ring: Ring,
}

#[typetag::serde]
impl Item for RingItem {
    /// When the item is used, equip the inner ring in the player.
    /// If the player was already wearing two rings, move the second one back
    /// to the inventory.
    fn apply(&mut self, game: &mut game::Game) {
        // In order to move out the inner ring (without having to change it to an Option)
        // replace its memory with a throw away Void ring
        let ring = std::mem::replace(&mut self.ring, Ring::Void);

        ring.equip_side_effect(&mut game.player);
        if let Some(removed) = game.player.rings.equip(ring) {
            removed.unequip_side_effect(&mut game.player);
            let key = removed.key();
            game.add_item(&key, Box::new(RingItem { ring: removed }));
        }
    }
}

// TODO add tests
// TODO make sure that all uses of these modified stats call the ring modified version
