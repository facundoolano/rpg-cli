use crate::game;
use crate::item::equipment::{Shield, Sword};
use crate::item::{equipment::Equipment, Escape, Item, Potion, Remedy};
use crate::randomizer::random;
use crate::randomizer::Randomizer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A chest is a bag of items that can be picked up by the hero.
/// It can randomly appear at a location upon inspection, or dropped
/// by the hero when they die.
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
        // To give the impression of "dynamic" chest contents, each content type
        // is randomized separately, and what's found is combined into a single
        // chest at the end
        let distance = &game.location.distance_from_home();
        let gold_chest = random().gold_chest(distance);
        let equipment_chest = random().equipment_chest(distance);
        let item_chest = random().item_chest(distance);

        let mut chest = Self::default();

        if gold_chest {
            chest.gold = random().gold_gained(game.player.level * 200)
        }

        if equipment_chest {
            let (sword, shield) = random_equipment(game.player.rounded_level());
            chest.sword = sword;
            chest.shield = shield;
        }

        if item_chest {
            chest.items = random_items(game.player.rounded_level());
        }

        // Return None instead of an empty chest if none was found
        if gold_chest || equipment_chest || item_chest {
            Some(chest)
        } else {
            None
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

    /// Add the elements of `other` to this chest
    pub fn extend(&mut self, mut other: Self) {
        // keep the best of each equipment
        if let Some(sword) = other.sword.take() {
            if sword.is_upgrade_from(&self.sword.as_ref()) {
                self.sword = Some(sword);
            }
        }

        if let Some(shield) = other.shield.take() {
            if shield.is_upgrade_from(&self.shield.as_ref()) {
                self.shield = Some(shield);
            }
        }

        // merge both item maps
        for (key, other_items) in other.items.drain() {
            let self_items = self.items.entry(key).or_default();
            self_items.extend(other_items);
        }

        self.gold += other.gold;
    }
}

// TODO consider using weighted random instead of these matches
fn random_equipment(level: i32) -> (Option<Sword>, Option<Shield>) {
    match random().range(15) {
        n if n < 8 => (Some(Sword::new(level)), None),
        n if n < 13 => (None, Some(Shield::new(level))),
        14 => (Some(Sword::new(level + 5)), None),
        _ => (None, Some(Shield::new(level + 5))),
    }
}

fn random_items(level: i32) -> HashMap<String, Vec<Box<dyn Item>>> {
    let mut map = HashMap::new();
    let potion = || Box::new(Potion::new(level));

    let (key, items): (&str, Vec<Box<dyn Item>>) = match random().range(15) {
        n if n < 7 => ("potion", vec![potion()]),
        n if n < 11 => ("potion", vec![potion(), potion()]),
        n if n < 13 => ("potion", vec![potion(), potion(), potion()]),
        13 => ("remedy", vec![Box::new(Remedy::new())]),
        _ => ("escape", vec![Box::new(Escape::new())]),
    };
    map.insert(key.to_string(), items);
    map
}

impl Default for Chest {
    fn default() -> Self {
        Self {
            gold: 0,
            sword: None,
            shield: None,
            items: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::equipment::{Shield, Sword};
    use crate::item::{Escape, Potion};

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

    #[test]
    fn test_merge() {
        let potions: Vec<Box<dyn Item>> = vec![Box::new(Potion::new(1)), Box::new(Potion::new(1))];
        let mut items = HashMap::new();
        items.insert("potion".to_string(), potions);
        let mut chest1 = Chest {
            items,
            sword: Some(Sword::new(1)),
            shield: Some(Shield::new(10)),
            gold: 100,
        };

        let potions: Vec<Box<dyn Item>> = vec![Box::new(Potion::new(1))];
        let escapes: Vec<Box<dyn Item>> = vec![Box::new(Escape::new())];
        let mut items = HashMap::new();
        items.insert("potion".to_string(), potions);
        items.insert("escape".to_string(), escapes);
        let chest2 = Chest {
            items,
            sword: Some(Sword::new(10)),
            shield: Some(Shield::new(1)),
            gold: 100,
        };

        chest1.extend(chest2);
        assert_eq!(200, chest1.gold);
        assert_eq!(10, chest1.sword.as_ref().unwrap().level());
        assert_eq!(10, chest1.shield.as_ref().unwrap().level());
        assert_eq!(3, chest1.items.get("potion").unwrap().len());
        assert_eq!(1, chest1.items.get("escape").unwrap().len());
    }
}
