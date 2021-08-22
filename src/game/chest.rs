use crate::game;
use crate::item::equipment::Equipment;
use crate::item::ring;
use crate::item::stone;
use crate::item::{Escape, Ether, Item, Potion, Remedy};
use crate::randomizer::random;
use crate::randomizer::Randomizer;
use rand::prelude::{IteratorRandom, SliceRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A chest is a bag of items that can be picked up by the hero.
/// It can randomly appear at a location upon inspection, or dropped
/// by the hero when they die.
#[derive(Serialize, Deserialize)]
pub struct Chest {
    items: HashMap<String, Vec<Box<dyn Item>>>,
    sword: Option<Equipment>,
    shield: Option<Equipment>,
    gold: i32,
}

impl Chest {
    /// Randomly generate a chest at the current location.
    pub fn generate(game: &mut game::Game) -> Option<Self> {
        // To give the impression of "dynamic" chest contents, each content type
        // is randomized separately, and what's found is combined into a single
        // chest at the end
        let distance = &game.location.distance_from_home();
        let gold_chest = random().gold_chest(distance);
        let equipment_chest = random().equipment_chest(distance);
        let item_chest = random().item_chest(distance);
        let ring_chest = random().ring_chest(distance);

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

        if ring_chest {
            if let Some(ring) = random_ring(game) {
                let key = ring.to_string();
                chest.items.insert(key, vec![Box::new(ring)]);
            }
        }

        // Return None instead of an empty chest if none was found
        if gold_chest || equipment_chest || item_chest {
            Some(chest)
        } else {
            None
        }
    }

    pub fn battle_loot(game: &mut game::Game) -> Option<Self> {
        // reuse item % from chests, but don't add extra gold
        // kind of hacky but does for now
        Self::generate(game).map(|mut c| {
            c.gold = 0;
            c
        })
    }

    /// Remove the gold, items and equipment from a hero and return them as a new chest.
    pub fn drop(game: &mut game::Game) -> Self {
        let mut items: HashMap<String, Vec<Box<dyn Item>>> = game.inventory.drain().collect();
        let sword = game.player.sword.take();
        let shield = game.player.shield.take();

        // equipped rings should be dropped as items
        if let Some(ring) = game.player.left_ring.take() {
            let key = ring.to_string();
            items.insert(key, vec![Box::new(ring)]);
        }
        if let Some(ring) = game.player.right_ring.take() {
            let key = ring.to_string();
            items.insert(key, vec![Box::new(ring)]);
        }
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
        if maybe_upgrade(&mut game.player.sword, &mut self.sword) {
            to_log.push(game.player.sword.as_ref().unwrap().to_string());
        }
        if maybe_upgrade(&mut game.player.shield, &mut self.shield) {
            to_log.push(game.player.sword.as_ref().unwrap().to_string());
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
        maybe_upgrade(&mut self.sword, &mut other.sword);
        maybe_upgrade(&mut self.shield, &mut other.shield);

        // merge both item maps
        for (key, other_items) in other.items.drain() {
            let self_items = self.items.entry(key).or_default();
            self_items.extend(other_items);
        }

        self.gold += other.gold;
    }
}

/// Upgrades current with the other equipment if it has a better level (or current is None).
/// Return whether there was an upgrade.
fn maybe_upgrade(current: &mut Option<Equipment>, other: &mut Option<Equipment>) -> bool {
    if let Some(shield) = other.take() {
        if shield.is_upgrade_from(current) {
            current.replace(shield);
            return true;
        }
    }
    false
}

fn random_equipment(level: i32) -> (Option<Equipment>, Option<Equipment>) {
    let mut rng = rand::thread_rng();

    vec![
        (100, (Some(Equipment::Sword(level)), None)),
        (80, (None, Some(Equipment::Shield(level)))),
        (30, (Some(Equipment::Sword(level + 5)), None)),
        (20, (None, Some(Equipment::Shield(level + 5)))),
        (1, (Some(Equipment::Sword(100)), None)),
    ]
    .choose_weighted_mut(&mut rng, |c| c.0)
    .unwrap()
    .to_owned()
    .1
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

fn random_ring(game: &mut game::Game) -> Option<ring::Ring> {
    let mut rng = rand::thread_rng();
    if let Some(ring) = game.ring_pool.iter().choose(&mut rng).cloned() {
        game.ring_pool.take(&ring)
    } else {
        None
    }
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
    use crate::item::equipment::Equipment;
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
        game.player.sword = Some(Equipment::Sword(1));
        game.player.shield = Some(Equipment::Shield(1));
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
        game.player.sword = Some(Equipment::Sword(1));
        game.player.shield = Some(Equipment::Shield(10));
        game.gold = 100;

        let mut tomb = Chest::drop(&mut game);

        // set some defaults for the new game before picking up
        let mut game = game::Game::new();
        game.add_item("potion", Box::new(Potion::new(1)));
        game.player.sword = Some(Equipment::Sword(5));
        game.player.shield = Some(Equipment::Shield(5));
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
            sword: Some(Equipment::Sword(1)),
            shield: Some(Equipment::Shield(10)),
            gold: 100,
        };

        let potions: Vec<Box<dyn Item>> = vec![Box::new(Potion::new(1))];
        let escapes: Vec<Box<dyn Item>> = vec![Box::new(Escape::new())];
        let mut items = HashMap::new();
        items.insert("potion".to_string(), potions);
        items.insert("escape".to_string(), escapes);
        let chest2 = Chest {
            items,
            sword: Some(Equipment::Sword(10)),
            shield: Some(Equipment::Shield(1)),
            gold: 100,
        };

        chest1.extend(chest2);
        assert_eq!(200, chest1.gold);
        assert_eq!(10, chest1.sword.as_ref().unwrap().level());
        assert_eq!(10, chest1.shield.as_ref().unwrap().level());
        assert_eq!(3, chest1.items.get("potion").unwrap().len());
        assert_eq!(1, chest1.items.get("escape").unwrap().len());
    }

    #[test]
    fn test_take_random_ring() {
        let mut game = game::Game::new();
        let total = game.ring_pool.len();
        assert!(total > 0);

        for i in 0..total {
            assert_eq!(total - i, game.ring_pool.len());
            assert!(random_ring(&mut game).is_some());
        }

        assert!(game.ring_pool.is_empty());
        assert!(random_ring(&mut game).is_none());
    }

    #[test]
    fn test_drop_equipped_rings() {
        let mut game = game::Game::new();
        game.add_item("potion", Box::new(Potion::new(1)));
        game.player.left_ring = Some(ring::Ring::Speed);
        game.player.right_ring = Some(ring::Ring::Magic);

        let mut chest = Chest::drop(&mut game);
        assert!(game.player.left_ring.is_none());
        assert!(game.player.right_ring.is_none());
        assert!(chest.items.get("spd-rng").is_some());
        assert!(chest.items.get("mag-rng").is_some());

        chest.pick_up(&mut game);
        assert!(game.inventory.contains_key("spd-rng"));
        assert!(game.inventory.contains_key("mag-rng"));
    }
}
