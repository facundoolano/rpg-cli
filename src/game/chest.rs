use crate::game;
use crate::item::equipment::{Shield, Sword};
use crate::item::{equipment::Equipment, Item, Potion};
use crate::randomizer::random;
use crate::randomizer::Randomizer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The tombstone is a bag of items left at the hero's dying location.
/// When the next hero visits that location, it can pick up the items.
#[derive(Serialize, Deserialize)]
pub struct Chest {
    items: HashMap<String, Vec<Box<dyn Item>>>,
    sword: Option<Sword>,
    shield: Option<Shield>,
    gold: i32,
}

impl Chest {
    /// Randomly generate a chest at the current location.
    pub fn generate(game: &game::Game) -> Option<Self> {
        // FIXME improve random generation logic
        // FIXME inlcude other items

        match random().range(6) {
            0 => {
                let gold = random().gold_gained(game.player.level * 200);
                Some(Self {
                    items: HashMap::new(),
                    sword: None,
                    shield: None,
                    gold,
                })
            }
            1 => {
                let potion = Box::new(Potion::new(game.player.level));
                let potions: Vec<Box<dyn Item>> = vec![potion];

                let mut items = HashMap::new();
                items.insert("potion".to_string(), potions);

                Some(Self {
                    items,
                    sword: None,
                    shield: None,
                    gold: 0,
                })
            }
            _ => None,
        }
    }

    /// Remove the gold, items and equipment from a hero and return them as a new chest.
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
            gold,
        }
    }

    /// Add the items of this chest to the current game/hero
    pub fn pick_up(&mut self, game: &mut game::Game) -> (Vec<String>, i32) {
        let mut to_log = Vec::new();

        // the equipment is picked up only if it's better than the current one
        if let Some(sword) = self.sword.take() {
            if sword.is_upgrade_from(&game.player.sword.as_ref()) {
                to_log.push(sword.to_string());
                game.player.sword = Some(sword);
            }
        }

        if let Some(shield) = self.shield.take() {
            if shield.is_upgrade_from(&game.player.shield.as_ref()) {
                to_log.push(shield.to_string());
                game.player.shield = Some(shield);
            }
        }

        // items and gold are always picked up
        for (name, items) in self.items.drain() {
            // this is kind of leaking logging logic but well
            to_log.push(format!("{}x{}", name, items.len()));

            for item in items {
                game.add_item(&name, item);
            }
        }

        game.gold += self.gold;
        (to_log, self.gold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::equipment::{Shield, Sword};
    use crate::item::Potion;

    #[test]
    fn test_empty_drop_pickup() {
        let mut game = game::Game::new();
        let mut tomb = Chest::drop(&mut game);

        assert_eq!(0, tomb.gold);
        assert!(tomb.sword.is_none());
        assert!(tomb.shield.is_none());
        assert!(tomb.items.is_empty());

        let mut game = game::Game::new();
        tomb.pick_up(&mut game);

        assert_eq!(0, game.gold);
        assert!(game.player.sword.is_none());
        assert!(game.player.shield.is_none());
        assert!(game.inventory().is_empty());
    }

    #[test]
    fn test_full_drop_pickup() {
        let mut game = game::Game::new();
        game.add_item("potion", Box::new(Potion::new(1)));
        game.add_item("potion", Box::new(Potion::new(1)));
        game.player.sword = Some(Sword::new(1));
        game.player.shield = Some(Shield::new(1));
        game.gold = 100;

        let mut tomb = Chest::drop(&mut game);

        assert_eq!(100, tomb.gold);
        assert!(tomb.sword.is_some());
        assert!(tomb.shield.is_some());
        assert_eq!(2, tomb.items.get("potion").unwrap().len());

        let mut game = game::Game::new();
        tomb.pick_up(&mut game);

        assert_eq!(100, game.gold);
        assert!(game.player.sword.is_some());
        assert!(game.player.shield.is_some());
        assert_eq!(2, *game.inventory().get("potion").unwrap());
    }

    #[test]
    fn test_pickup_extends() {
        let mut game = game::Game::new();
        game.add_item("potion", Box::new(Potion::new(1)));
        game.add_item("potion", Box::new(Potion::new(1)));
        game.player.sword = Some(Sword::new(1));
        game.player.shield = Some(Shield::new(10));
        game.gold = 100;

        let mut tomb = Chest::drop(&mut game);

        // set some defaults for the new game before picking up
        let mut game = game::Game::new();
        game.add_item("potion", Box::new(Potion::new(1)));
        game.player.sword = Some(Sword::new(5));
        game.player.shield = Some(Shield::new(5));
        game.gold = 50;

        tomb.pick_up(&mut game);

        assert_eq!(150, game.gold);

        // the sword was upgrade, picked it up
        assert_eq!(5, game.player.sword.as_ref().unwrap().level());

        // the shield was downgrade, kept the current one
        assert_eq!(10, game.player.shield.as_ref().unwrap().level());

        assert_eq!(3, *game.inventory().get("potion").unwrap());
    }
}
