use super::Item;
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

    fn apply(&self, base: i32, ring: Ring, mult: f64) -> i32 {
        let lmult = match &self.left {
            Some(left_ring) if ring == *left_ring => mult,
            _ => 0.0,
        };
        let rmult = match &self.right {
            Some(right_ring) if ring == *right_ring => mult,
            _ => 0.0,
        };

        (base as f64 * (lmult + rmult)).round() as i32
    }

    pub fn attack(&self, strength: i32) -> i32 {
        self.apply(strength, Ring::Attack, 0.5)
    }

    pub fn deffense(&self, strength: i32) -> i32 {
        self.apply(strength, Ring::Deffense, 0.5)
    }

    pub fn speed(&self, strength: i32) -> i32 {
        self.apply(strength, Ring::Deffense, 0.5)
    }

    pub fn magic(&self, base: i32) -> i32 {
        self.apply(base, Ring::Magic, 0.5)
    }

    pub fn mp(&self, base: i32) -> i32 {
        self.apply(base, Ring::MP, 0.5)
    }

    pub fn hp(&self, base: i32) -> i32 {
        self.apply(base, Ring::HP, 0.5)
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
            Ring::Attack => "attack",
            Ring::Deffense => "deffense",
            Ring::Speed => "speed",
            Ring::Magic => "magic",
            Ring::MP => "mp",
            Ring::HP => "hp",
        }
    }
}

// TODO add tests
// TODO make sure that all uses of these modified stats call the ring modified version
