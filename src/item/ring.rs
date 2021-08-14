use super::Item;
use crate::game;
use serde::{Deserialize, Serialize};

/// TODO
#[typetag::serde(tag = "type")]
pub trait Ring {
    fn key(&self) -> String;
}

/// TODO
#[derive(Serialize, Deserialize)]
pub struct RingPair {
    left: Option<Box<dyn Ring>>,
    right: Option<Box<dyn Ring>>,
}

#[derive(Serialize, Deserialize)]
pub struct RingItem {
    ring: Box<dyn Ring>,
}

#[typetag::serde]
impl Item for RingItem {
    /// When the item is used, equip the inner ring in the player.
    /// If the player was already wearing two rings, move the second one back
    /// to the inventory
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Void;

#[typetag::serde]
impl Ring for Void {
    fn key(&self) -> String {
        String::from("void")
    }
}
