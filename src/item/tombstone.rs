use super::Item;
use crate::game;
use super::equipment::{Shield, Sword};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The tombstone is a bag of items left at the hero's dying location.
/// When the next hero visits that location, it can pick up the items.
#[derive(Serialize, Deserialize)]
pub struct Tombstone {
    items: HashMap<String, Vec<Box<dyn Item>>>,
    pub sword: Option<Sword>,
    pub shield: Option<Shield>,
    pub gold: i32,
}

impl Tombstone {
    /// Dump the equipment, items and gold from a hero.
    pub fn drop(game: &mut game::Game) -> Self {
        todo!();
    }

    /// Add the items of the tombstone to the current game
    pub fn pick_up(&mut self, game: &mut game::Game) {
        // items and gold are always picked up
        // the equipment is picked up only if it's better than the current one
        todo!();
    }
}
