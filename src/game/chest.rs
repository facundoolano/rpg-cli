use crate::game;
use crate::item::equipment::Equipment;
use crate::item::key::Key;
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
    items: Vec<Box<dyn Item>>,
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

        if ring_chest {
            // Because of the ring pool (only one instance per ring type), it's
            // easier to handle this case separate from the rest of the items
            // --only remove from the pool if we are positive a ring should be
            // be included in the chest
            if let Some(ring) = random_ring(game) {
                chest.items.push(Box::new(ring));
            }
        }

        // Items should be more frequent and can be multiple
        let mut item_chest = false;
        for _ in 0..3 {
            if random().item_chest(distance) {
                item_chest = true;
                let item = random_item(game.player.rounded_level());
                chest.items.push(item);
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
        let items: HashMap<Key, Vec<Box<dyn Item>>> = game.inventory.drain().collect();
        let mut items: Vec<Box<dyn Item>> = items.into_values().flatten().collect();
        let sword = game.player.sword.take();
        let shield = game.player.shield.take();

        // equipped rings should be dropped as items
        if let Some(ring) = game.player.left_ring.take() {
            items.push(Box::new(ring));
        }
        if let Some(ring) = game.player.right_ring.take() {
            items.push(Box::new(ring));
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
    /// Return a picked up (item counts, gold) tuple.
    pub fn pick_up(&mut self, game: &mut game::Game) -> (HashMap<Key, i32>, i32) {
        let mut item_counts = HashMap::new();

        // the equipment is picked up only if it's better than the current one
        if maybe_upgrade(&mut game.player.sword, &mut self.sword) {
            item_counts.insert(Key::Sword, 1);
        }
        if maybe_upgrade(&mut game.player.shield, &mut self.shield) {
            item_counts.insert(Key::Shield, 1);
        }

        // items and gold are always picked up
        for item in self.items.drain(..) {
            *item_counts.entry(item.key()).or_insert(0) += 1;
            game.add_item(item);
        }

        game.gold += self.gold;
        (item_counts, self.gold)
    }

    /// Add the elements of `other` to this chest
    pub fn extend(&mut self, mut other: Self) {
        // keep the best of each equipment
        maybe_upgrade(&mut self.sword, &mut other.sword);
        maybe_upgrade(&mut self.shield, &mut other.shield);
        self.items.extend(other.items.drain(..));
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
        (100, (Some(Equipment::sword(level)), None)),
        (80, (None, Some(Equipment::shield(level)))),
        (30, (Some(Equipment::sword(level + 5)), None)),
        (20, (None, Some(Equipment::shield(level + 5)))),
        (1, (Some(Equipment::sword(100)), None)),
    ]
    .choose_weighted_mut(&mut rng, |c| c.0)
    .unwrap()
    .to_owned()
    .1
}

/// Return a weigthed random item.
fn random_item(level: i32) -> Box<dyn Item> {
    let mut choices: Vec<(i32, Box<dyn Item>)> = vec![
        (150, Box::new(Potion::new(level))),
        (10, Box::new(Remedy::new())),
        (10, Box::new(Escape::new())),
        (50, Box::new(Ether::new(level))),
        (10, Box::new(stone::Health)),
        (10, Box::new(stone::Magic)),
        (10, Box::new(stone::Power)),
        (10, Box::new(stone::Speed)),
        (5, Box::new(stone::Level)),
    ];

    // make a separate vec with enumerated weights, then remove from the item vec
    // with the resulting index
    let indexed_weights: Vec<_> = choices.iter().map(|(w, _)| w).enumerate().collect();

    let mut rng = rand::thread_rng();
    let index = indexed_weights
        .choose_weighted(&mut rng, |c| c.1)
        .unwrap()
        .0;
    choices.remove(index).1
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
            items: Vec::new(),
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
        game.add_item(Box::new(Potion::new(1)));
        game.add_item(Box::new(Potion::new(1)));
        game.player.sword = Some(Equipment::sword(1));
        game.player.shield = Some(Equipment::shield(1));
        game.gold = 100;

        let mut tomb = Chest::drop(&mut game);

        assert_eq!(100, tomb.gold);
        assert!(tomb.sword.is_some());
        assert!(tomb.shield.is_some());
        assert_eq!(2, tomb.items.len());

        let mut game = game::Game::new();
        tomb.pick_up(&mut game);

        assert_eq!(100, game.gold);
        assert!(game.player.sword.is_some());
        assert!(game.player.shield.is_some());
        assert_eq!(2, *game.inventory().get(&Key::Potion).unwrap());
    }

    #[test]
    fn test_pickup_extends() {
        let mut game = game::Game::new();
        game.add_item(Box::new(Potion::new(1)));
        game.add_item(Box::new(Potion::new(1)));
        game.player.sword = Some(Equipment::sword(1));
        game.player.shield = Some(Equipment::shield(10));
        game.gold = 100;

        let mut tomb = Chest::drop(&mut game);

        // set some defaults for the new game before picking up
        let mut game = game::Game::new();
        game.add_item(Box::new(Potion::new(1)));
        game.player.sword = Some(Equipment::sword(5));
        game.player.shield = Some(Equipment::shield(5));
        game.gold = 50;

        tomb.pick_up(&mut game);

        assert_eq!(150, game.gold);

        // the sword was upgrade, picked it up
        assert_eq!(5, game.player.sword.as_ref().unwrap().level());

        // the shield was downgrade, kept the current one
        assert_eq!(10, game.player.shield.as_ref().unwrap().level());

        assert_eq!(3, *game.inventory().get(&Key::Potion).unwrap());
    }

    #[test]
    fn test_merge() {
        let items: Vec<Box<dyn Item>> = vec![Box::new(Potion::new(1)), Box::new(Potion::new(1))];
        let mut chest1 = Chest {
            items,
            sword: Some(Equipment::sword(1)),
            shield: Some(Equipment::shield(10)),
            gold: 100,
        };

        let items: Vec<Box<dyn Item>> = vec![Box::new(Potion::new(1)), Box::new(Escape::new())];
        let chest2 = Chest {
            items,
            sword: Some(Equipment::sword(10)),
            shield: Some(Equipment::shield(1)),
            gold: 100,
        };

        chest1.extend(chest2);
        assert_eq!(200, chest1.gold);
        assert_eq!(10, chest1.sword.as_ref().unwrap().level());
        assert_eq!(10, chest1.shield.as_ref().unwrap().level());
        let item_keys = chest1.items.iter().map(|i| i.key()).collect::<Vec<_>>();
        assert_eq!(
            vec![Key::Potion, Key::Potion, Key::Potion, Key::Escape],
            item_keys
        );
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
        game.add_item(Box::new(Potion::new(1)));
        game.player.left_ring = Some(ring::Ring::Speed);
        game.player.right_ring = Some(ring::Ring::Magic);

        let mut chest = Chest::drop(&mut game);
        assert!(game.player.left_ring.is_none());
        assert!(game.player.right_ring.is_none());
        let item_keys = chest.items.iter().map(|i| i.key()).collect::<Vec<_>>();
        assert_eq!(
            vec![
                Key::Potion,
                Key::Ring(ring::Ring::Speed),
                Key::Ring(ring::Ring::Magic)
            ],
            item_keys
        );

        chest.pick_up(&mut game);
        assert!(game.inventory.contains_key(&Key::Ring(ring::Ring::Speed)));
        assert!(game.inventory.contains_key(&Key::Ring(ring::Ring::Magic)));
    }
}
