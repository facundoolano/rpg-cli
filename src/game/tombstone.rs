use crate::item::Item;
use crate::game;
use crate::item::equipment::{Shield, Sword};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The tombstone is a bag of items left at the hero's dying location.
/// When the next hero visits that location, it can pick up the items.
#[derive(Serialize, Deserialize)]
pub struct Tombstone {
    items: HashMap<String, Vec<Box<dyn Item>>>,
    sword: Option<Sword>,
    shield: Option<Shield>,
    gold: i32,
}

impl Tombstone {

    /// Dump the equipment, items and gold from a hero.
    pub fn drop(game: &mut game::Game) -> Self {
        let sword = game.player.sword.take();
        let shield = game.player.shield.take();
        let items = game.inventory.drain().collect();
        let gold = game.gold;
        game.gold = 0;

        Self {
            items,
            sword,
            shield,
            gold
        }
    }

    /// Add the items of the tombstone to the current game
    pub fn pick_up(&mut self, game: &mut game::Game) {
        // items and gold are always picked up
        // the equipment is picked up only if it's better than the current one
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_drop_pickup() {

    }

    #[test]
    fn test_full_drop_pickup() {

    }
}
