use super::Item;
use crate::game;
use serde::{Deserialize, Serialize};

#[typetag::serde(tag = "type")]
pub trait Ring {}

#[derive(Serialize, Deserialize)]
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

    fn equip(&mut self, ring: Box<dyn Ring>) {
        // FIXME swap etc
        self.left = Some(ring);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Void;

#[typetag::serde]
impl Ring for Void {}

#[typetag::serde]
impl Item for Void {
    fn apply(&self, game: &mut game::Game) {
        game.player.rings.equip(Box::new(self.clone()))
    }
}
