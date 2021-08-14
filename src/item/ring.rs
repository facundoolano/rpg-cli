use super::Item;
use crate::game;
use serde::{Deserialize, Serialize};

/// Rings are a kind of equipment that produce arbitrary effects hooked in
/// different places of the game, e.g. increase a stat, double gold gained, etc.
#[typetag::serde(tag = "type")]
pub trait Ring {
    fn key(&self) -> String;
}

/// The character is allowed to hold two rings.
/// The ring pair struct is used to hold the rings that the character is wearing,
/// handling the equipping and calculating the net combined effect of the two rings.
#[derive(Serialize, Deserialize, Default)]
pub struct RingPair {
    left: Option<Box<dyn Ring>>,
    right: Option<Box<dyn Ring>>,
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
    fn equip(&mut self, ring: Box<dyn Ring>) -> Option<Box<dyn Ring>> {
        if let Some(old_left) = self.left.replace(ring) {
            return self.right.replace(old_left);
        }
        None
    }
}

/// RingItem is a wrapper that lets the rings be added to the inventory and
/// used them to equip them.
#[derive(Serialize, Deserialize)]
pub struct RingItem {
    ring: Box<dyn Ring>,
}

#[typetag::serde]
impl Item for RingItem {
    /// When the item is used, equip the inner ring in the player.
    /// If the player was already wearing two rings, move the second one back
    /// to the inventory.
    fn apply(&mut self, game: &mut game::Game) {
        // In order to move out the inner ring (without having to change it to an Option)
        // replace its memory with a throw away Void ring
        let ring = std::mem::replace(&mut self.ring, Box::new(Void));

        if let Some(removed) = game.player.rings.equip(ring) {
            let key = removed.key();
            game.add_item(&key, Box::new(RingItem { ring: removed }));
        }
    }
}



#[derive(Serialize, Deserialize)]
pub struct Void;

#[typetag::serde]
impl Ring for Void {
    fn key(&self) -> String {
        String::from("void")
    }
}
