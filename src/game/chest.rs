use crate::game;
use crate::item::equipment::{Equipment, Weapon};
use crate::item::stone;
use crate::item::{Escape, Ether, Item, Potion, Remedy};
use crate::randomizer::random;
use crate::randomizer::Randomizer;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A chest is a bag of items that can be picked up by the hero.
/// It can randomly appear at a location upon inspection, or dropped
/// by the hero when they die.
#[derive(Serialize, Deserialize)]
pub struct Chest {
    items: HashMap<String, Vec<Box<dyn Item>>>,
    equip: Equipment,
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
            chest.equip = random_equipment(game.player.rounded_level());
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

    pub fn battle_loot(game: &game::Game) -> Option<Self> {
        // reuse item % from chests, but don't add extra gold
        // kind of hacky but does for now
        Self::generate(game).map(|mut c| {
            c.gold = 0;
            c
        })
    }

    /// Remove the gold, items and equipment from a hero and return them as a new chest.
    pub fn drop(game: &mut game::Game) -> Self {
        let equip = std::mem::take(&mut game.player.equip);
        let items = game.inventory.drain().collect();
        let gold = game.gold;
        game.gold = 0;

        Self { items, equip, gold }
    }

    /// Add the items of this chest to the current game/hero
    pub fn pick_up(&mut self, game: &mut game::Game) -> (Vec<String>, i32) {
        let mut to_log = Vec::new();

        // the equipment is picked up only if it's better than the current one
        let (upgraded_sword, upgraded_shield) = game.player.equip.upgrade(&mut self.equip);
        if upgraded_sword {
            to_log.push(game.player.equip.sword.as_ref().unwrap().to_string());
        }
        if upgraded_shield {
            to_log.push(game.player.equip.sword.as_ref().unwrap().to_string());
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
        self.equip.upgrade(&mut other.equip);

        // merge both item maps
        for (key, other_items) in other.items.drain() {
            let self_items = self.items.entry(key).or_default();
            self_items.extend(other_items);
        }

        self.gold += other.gold;
    }
}

fn random_equipment(level: i32) -> Equipment {
    let mut rng = rand::thread_rng();

    let (sword, shield) = vec![
        (100, (Some(Weapon::Sword(level)), None)),
        (80, (None, Some(Weapon::Shield(level)))),
        (30, (Some(Weapon::Sword(level + 5)), None)),
        (20, (None, Some(Weapon::Shield(level + 5)))),
        (1, (Some(Weapon::Sword(100)), None)),
    ]
    .choose_weighted_mut(&mut rng, |c| c.0)
    .unwrap()
    .to_owned()
    .1;

    Equipment {
        sword,
        shield,
        left_ring: None,
        right_ring: None,
    }
}

type WeightedItems = (i32, &'static str, Vec<Box<dyn Item>>);

fn random_items(level: i32) -> HashMap<String, Vec<Box<dyn Item>>> {
    let potion = || Box::new(Potion::new(level));

    let mut choices: Vec<WeightedItems> = vec![
        (100, "potion", vec![potion()]),
        (30, "potion", vec![potion(), potion()]),
        (10, "potion", vec![potion(), potion(), potion()]),
        (10, "remedy", vec![Box::new(Remedy::new())]),
        (10, "escape", vec![Box::new(Escape::new())]),
        (50, "ether", vec![Box::new(Ether::new(level))]),
        (10, "hp-stone", vec![Box::new(stone::Health)]),
        (10, "mp-stone", vec![Box::new(stone::Magic)]),
        (10, "str-stone", vec![Box::new(stone::Power)]),
        (10, "spd-stone", vec![Box::new(stone::Speed)]),
        (5, "lvl-stone", vec![Box::new(stone::Level)]),
    ];

    let mut rng = rand::thread_rng();
    let (_, key, items) = choices.choose_weighted_mut(&mut rng, |c| c.0).unwrap();
    let items = items.drain(..).collect();
    let mut map = HashMap::new();
    map.insert(key.to_string(), items);
    map
}

impl Default for Chest {
    fn default() -> Self {
        Self {
            gold: 0,
            equip: Equipment::new(),
            items: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::equipment::Weapon;
    use crate::item::{Escape, Potion};

    #[test]
    fn test_empty_drop_pickup() {
        let mut game = game::Game::new();
        let mut tomb = Chest::drop(&mut game);

        assert_eq!(0, tomb.gold);
        assert!(tomb.equip.sword.is_none());
        assert!(tomb.equip.shield.is_none());
        assert!(tomb.items.is_empty());

        let mut game = game::Game::new();
        tomb.pick_up(&mut game);

        assert_eq!(0, game.gold);
        assert!(game.player.equip.sword.is_none());
        assert!(game.player.equip.shield.is_none());
        assert!(game.inventory().is_empty());
    }

    #[test]
    fn test_full_drop_pickup() {
        let mut game = game::Game::new();
        game.add_item("potion", Box::new(Potion::new(1)));
        game.add_item("potion", Box::new(Potion::new(1)));
        game.player.equip.sword = Some(Weapon::Sword(1));
        game.player.equip.shield = Some(Weapon::Shield(1));
        game.gold = 100;

        let mut tomb = Chest::drop(&mut game);

        assert_eq!(100, tomb.gold);
        assert!(tomb.equip.sword.is_some());
        assert!(tomb.equip.shield.is_some());
        assert_eq!(2, tomb.items.get("potion").unwrap().len());

        let mut game = game::Game::new();
        tomb.pick_up(&mut game);

        assert_eq!(100, game.gold);
        assert!(game.player.equip.sword.is_some());
        assert!(game.player.equip.shield.is_some());
        assert_eq!(2, *game.inventory().get("potion").unwrap());
    }

    #[test]
    fn test_pickup_extends() {
        let mut game = game::Game::new();
        game.add_item("potion", Box::new(Potion::new(1)));
        game.add_item("potion", Box::new(Potion::new(1)));
        game.player.equip.sword = Some(Weapon::Sword(1));
        game.player.equip.shield = Some(Weapon::Shield(10));
        game.gold = 100;

        let mut tomb = Chest::drop(&mut game);

        // set some defaults for the new game before picking up
        let mut game = game::Game::new();
        game.add_item("potion", Box::new(Potion::new(1)));
        game.player.equip.sword = Some(Weapon::Sword(5));
        game.player.equip.shield = Some(Weapon::Shield(5));
        game.gold = 50;

        tomb.pick_up(&mut game);

        assert_eq!(150, game.gold);

        // the sword was upgrade, picked it up
        assert_eq!(5, game.player.equip.sword.as_ref().unwrap().level());

        // the shield was downgrade, kept the current one
        assert_eq!(10, game.player.equip.shield.as_ref().unwrap().level());

        assert_eq!(3, *game.inventory().get("potion").unwrap());
    }

    #[test]
    fn test_merge() {
        let potions: Vec<Box<dyn Item>> = vec![Box::new(Potion::new(1)), Box::new(Potion::new(1))];
        let mut items = HashMap::new();
        items.insert("potion".to_string(), potions);
        let mut chest1 = Chest {
            items,
            equip: Equipment {
                sword: Some(Weapon::Sword(1)),
                shield: Some(Weapon::Shield(10)),
                left_ring: None,
                right_ring: None,
            },
            gold: 100,
        };

        let potions: Vec<Box<dyn Item>> = vec![Box::new(Potion::new(1))];
        let escapes: Vec<Box<dyn Item>> = vec![Box::new(Escape::new())];
        let mut items = HashMap::new();
        items.insert("potion".to_string(), potions);
        items.insert("escape".to_string(), escapes);
        let chest2 = Chest {
            items,
            equip: Equipment {
                sword: Some(Weapon::Sword(10)),
                shield: Some(Weapon::Shield(1)),
                left_ring: None,
                right_ring: None,
            },
            gold: 100,
        };

        chest1.extend(chest2);
        assert_eq!(200, chest1.gold);
        assert_eq!(10, chest1.equip.sword.as_ref().unwrap().level());
        assert_eq!(10, chest1.equip.shield.as_ref().unwrap().level());
        assert_eq!(3, chest1.items.get("potion").unwrap().len());
        assert_eq!(1, chest1.items.get("escape").unwrap().len());
    }
}
