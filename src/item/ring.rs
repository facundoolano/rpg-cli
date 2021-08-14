use super::Item;
use crate::game;
use serde::{Deserialize, Serialize};

/// Rings are a kind of equipment that produce arbitrary effects hooked in
/// different places of the game, e.g. increase a stat, double gold gained, etc.
#[derive(Serialize, Deserialize)]
pub enum Ring {
    Void,
    AttackRing,
    DeffenseRing,
    MagicRing,
    MPRing,
    HPRing,
}

/// The character is allowed to hold two rings.
/// The ring pair struct is used to hold the rings that the character is wearing,
/// handling the equipping and calculating the net combined effect of the two rings.
#[derive(Serialize, Deserialize, Default)]
pub struct RingPair {
    left: Option<Ring>,
    right: Option<Ring>,
}

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

    fn apply_mult(&self, base: i32, fun: fn(&Ring) -> f64) -> i32 {
        let base = base as f64;
        let mult = self.left.as_ref().map_or(1.0, fun) + self.right.as_ref().map_or(1.0, fun);
        (base * mult).round() as i32
    }

    pub fn attack(&self, base: i32) -> i32 {
        self.apply_mult(base, Ring::attack_mult)
    }

    pub fn deffense(&self, base: i32) -> i32 {
        self.apply_mult(base, Ring::deffense_mult)
    }

    pub fn magic(&self, base: i32) -> i32 {
        self.apply_mult(base, Ring::magic_mult)
    }

    pub fn mp(&self, base: i32) -> i32 {
        self.apply_mult(base, Ring::mp_mult)
    }

    pub fn hp(&self, base: i32) -> i32 {
        self.apply_mult(base, Ring::hp_mult)
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

        if let Some(removed) = game.player.rings.equip(ring) {
            let key = removed.key();
            game.add_item(&key, Box::new(RingItem { ring: removed }));
        }
    }
}

impl Ring {
    fn key(&self) -> &'static str {
        match self {
            Ring::Void => "void",
            Ring::AttackRing => "attack",
            Ring::DeffenseRing => "deffense",
            Ring::MagicRing => "magic",
            Ring::MPRing => "mp",
            Ring::HPRing => "hp",
        }
    }

    fn attack_mult(&self) -> f64 {
        if let Ring::AttackRing = self {
            1.5
        } else {
            1.0
        }
    }

    fn deffense_mult(&self) -> f64 {
        1.0
    }

    fn magic_mult(&self) -> f64 {
        1.0
    }

    fn mp_mult(&self) -> f64 {
        1.0
    }

    fn hp_mult(&self) -> f64 {
        1.0
    }
}
