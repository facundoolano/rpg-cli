extern crate dirs;

use crate::character;
use crate::character::Character;
use crate::event::Event;
use crate::item::key::Key;
use crate::item::ring::Ring;
use crate::item::Item;
use crate::location::Location;
use crate::quest::QuestList;
use crate::randomizer::random;
use crate::randomizer::Randomizer;
use anyhow::{bail, Result};
use chest::Chest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub mod battle;
pub mod chest;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Game {
    pub player: Character,
    pub location: Location,
    pub gold: i32,
    pub quests: QuestList,
    pub inventory: HashMap<Key, Vec<Box<dyn Item>>>,

    /// Chest left at the location where the player dies.
    pub tombstones: HashMap<String, Chest>,

    /// There's one instance of each type of ring in the game.
    /// This set starts with all rings and they are moved to the inventory as
    /// they are found in chests.
    pub ring_pool: HashSet<Ring>,

    /// Locations where chest have already been looked for, and therefore
    /// can't be found again.
    inspected: HashSet<Location>,
}

impl Game {
    pub fn new() -> Self {
        let quests = QuestList::new();
        Self {
            location: Location::home(),
            player: Character::player(),
            gold: 0,
            inventory: HashMap::new(),
            tombstones: HashMap::new(),
            inspected: HashSet::new(),
            quests,
            ring_pool: Ring::set(),
        }
    }

    /// Remove the game data and reset this reference.
    /// Progress is preserved across games.
    pub fn reset(&mut self) {
        let mut new_game = Self::new();
        // preserve tombstones and quests across hero's lifes
        std::mem::swap(&mut new_game.tombstones, &mut self.tombstones);
        std::mem::swap(&mut new_game.quests, &mut self.quests);
        std::mem::swap(&mut new_game.ring_pool, &mut self.ring_pool);

        // remember last selected class
        new_game
            .player
            .change_class(&self.player.class.name)
            .unwrap_or_default();

        // replace the current, finished game with the new one
        *self = new_game;

        Event::emit(self, Event::GameReset);
    }

    /// Move the hero's location towards the given destination, one directory
    /// at a time, with some chance of enemies appearing on each one.
    pub fn go_to(
        &mut self,
        dest: &Location,
        run: bool,
        bribe: bool,
    ) -> Result<(), character::Dead> {
        while self.location != *dest {
            self.visit(self.location.go_to(dest))?;

            if !self.location.is_home() {
                if let Some(mut enemy) = self.maybe_spawn_enemy() {
                    return self.maybe_battle(&mut enemy, run, bribe);
                }
            }
        }
        Ok(())
    }

    /// Look for chests and tombstones at the current location.
    /// Remembers previously visited locations for consistency.
    pub fn inspect(&mut self) {
        let maybe_tomb = self.tombstones.remove(&self.location.to_string());
        self.pick_up_chest(maybe_tomb, true);

        if !self.inspected.contains(&self.location) {
            self.inspected.insert(self.location.clone());
            let chest = Chest::generate(self);
            self.pick_up_chest(chest, false);
        }
    }

    fn pick_up_chest(&mut self, maybe_chest: Option<Chest>, is_tombstone: bool) {
        if let Some(mut chest) = maybe_chest {
            let (items, gold) = chest.pick_up(self);
            Event::emit(
                self,
                Event::ChestFound {
                    items,
                    gold,
                    is_tombstone,
                },
            );
        }
    }

    /// Set the hero's location to the one given, and apply related side effects.
    pub fn visit(&mut self, location: Location) -> Result<(), character::Dead> {
        self.location = location;
        if self.location.is_home() {
            let (recovered_hp, recovered_mp, healed) = self.player.restore();
            Event::emit(
                self,
                Event::Heal {
                    item: None,
                    recovered_hp,
                    recovered_mp,
                    healed,
                },
            );
        }

        // In location is home, already healed of negative status
        self.apply_status_effects()
    }

    /// Player takes damage from status_effects, if any.
    fn apply_status_effects(&mut self) -> Result<(), character::Dead> {
        let (hp, mp) = self.player.apply_status_effects()?;
        Event::emit(
            self,
            Event::StatusEffect {
                enemy: None,
                hp,
                mp,
            },
        );
        Ok(())
    }

    /// Set the current location to home, and apply related side-effects
    pub fn visit_home(&mut self) {
        self.visit(Location::home()).unwrap_or_default();
    }

    pub fn add_item(&mut self, item: Box<dyn Item>) {
        let entry = self.inventory.entry(item.key()).or_insert_with(Vec::new);
        entry.push(item);
    }

    pub fn use_item(&mut self, name: Key) -> Result<()> {
        // get all items of that type and use one
        // if there are no remaining, drop the type from the inventory
        if let Some(mut items) = self.inventory.remove(&name) {
            if let Some(mut item) = items.pop() {
                item.apply(self);
                Event::emit(
                    self,
                    Event::ItemUsed {
                        item: name.to_string(),
                    },
                );
            }

            if !items.is_empty() {
                self.inventory.insert(name, items);
            }

            Ok(())
        } else if let Some(ring) = self.player.unequip_ring(&name) {
            // Rings are a special case of item in that they can be "used" while being
            // equipped, that is, while not being in the inventory.
            // The effect of using them is unequipping them.
            // This bit of complexity enables a cleaner command api.
            self.add_item(Box::new(ring));
            Ok(())
        } else {
            bail!("Item not found.")
        }
    }

    pub fn inventory(&self) -> HashMap<&Key, usize> {
        self.inventory
            .iter()
            .map(|(k, v)| (k, v.len()))
            .collect::<HashMap<&Key, usize>>()
    }

    pub fn change_class(&mut self, name: &str) -> Result<()> {
        if !self.location.is_home() {
            bail!("Class change is only allowed at home.")
        } else if let Ok(lost_xp) = self.player.change_class(name) {
            Event::emit(self, Event::ClassChanged { lost_xp });
            Ok(())
        } else {
            bail!("Unknown class name.");
        }
    }

    pub fn maybe_spawn_enemy(&mut self) -> Option<Character> {
        if self.player.enemies_evaded() {
            return None;
        }

        let distance = self.location.distance_from_home();
        if random().should_enemy_appear(&distance) {
            let enemy = character::enemy::at(&self.location, &self.player);

            Event::emit(self, Event::EnemyAppears { enemy: &enemy });
            Some(enemy)
        } else {
            None
        }
    }

    pub fn maybe_battle(
        &mut self,
        enemy: &mut Character,
        run: bool,
        bribe: bool,
    ) -> Result<(), character::Dead> {
        // don't attempt bribe and run in the same turn
        if bribe {
            if self.bribe(enemy) {
                return Ok(());
            }
        } else if run && self.run_away(enemy) {
            return Ok(());
        }

        self.battle(enemy)
    }

    fn bribe(&mut self, enemy: &Character) -> bool {
        let bribe_cost = gold_gained(self.player.level, enemy.level) / 2;

        if self.gold >= bribe_cost && random().bribe_succeeds() {
            self.gold -= bribe_cost;
            Event::emit(self, Event::Bribe { cost: bribe_cost });
            return true;
        };
        Event::emit(self, Event::Bribe { cost: 0 });
        false
    }

    fn run_away(&mut self, enemy: &Character) -> bool {
        let success = random().run_away_succeeds(
            self.player.level,
            enemy.level,
            self.player.speed(),
            enemy.speed(),
        );
        Event::emit(self, Event::RunAway { success });
        success
    }

    fn battle(&mut self, enemy: &mut Character) -> Result<(), character::Dead> {
        match battle::run(self, enemy) {
            Ok(xp) => {
                let gold = gold_gained(self.player.level, enemy.level);
                self.gold += gold;
                let levels_up = self.player.add_experience(xp);

                let reward_items = Chest::battle_loot(self)
                    .map_or(HashMap::new(), |mut chest| chest.pick_up(self).0);

                Event::emit(
                    self,
                    Event::BattleWon {
                        enemy,
                        location: self.location.clone(),
                        xp,
                        levels_up,
                        gold,
                        items: reward_items,
                    },
                );

                if levels_up > 0 {
                    Event::emit(
                        self,
                        Event::LevelUp {
                            count: levels_up,
                            current: self.player.level,
                            class: self.player.name(),
                        },
                    )
                }

                Ok(())
            }
            Err(character::Dead) => {
                // Drop hero items in the location. If there was a previous tombstone
                // merge the contents of both chests
                let mut tombstone = Chest::drop(self);
                let location = self.location.to_string();
                if let Some(previous) = self.tombstones.remove(&location) {
                    tombstone.extend(previous);
                }
                self.tombstones.insert(location, tombstone);

                Event::emit(self, Event::BattleLost);
                Err(character::Dead)
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

fn gold_gained(player_level: i32, enemy_level: i32) -> i32 {
    let level = std::cmp::max(1, enemy_level - player_level);
    random().gold_gained(level * 50)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item;

    #[test]
    fn test_inventory() {
        let mut game = Game::new();

        assert_eq!(0, game.inventory().len());

        let potion = item::Potion::new(1);
        game.add_item(Box::new(potion));
        assert_eq!(1, game.inventory().len());
        assert_eq!(1, *game.inventory().get(&Key::Potion).unwrap());

        let potion = item::Potion::new(1);
        game.add_item(Box::new(potion));
        assert_eq!(1, game.inventory().len());
        assert_eq!(2, *game.inventory().get(&Key::Potion).unwrap());

        game.player.current_hp -= 3;
        assert_ne!(game.player.max_hp(), game.player.current_hp);

        assert!(game.use_item(Key::Potion).is_ok());

        // check it actually restores the hp
        assert_eq!(game.player.max_hp(), game.player.current_hp);

        // check item was consumed
        assert_eq!(1, game.inventory().len());
        assert_eq!(1, *game.inventory().get(&Key::Potion).unwrap());

        assert!(game.use_item(Key::Potion).is_ok());
        assert_eq!(0, game.inventory().len());
        assert!(game.use_item(Key::Potion).is_err());
    }

    #[test]
    fn test_ring_equip() {
        let mut game = Game::new();

        assert!(game.player.left_ring.is_none());
        assert!(game.player.right_ring.is_none());

        game.add_item(Box::new(Ring::Void));
        game.add_item(Box::new(Ring::Void));
        game.add_item(Box::new(Ring::Void));
        assert_eq!(3, *game.inventory().get(&Key::Ring(Ring::Void)).unwrap());

        game.use_item(Key::Ring(Ring::Void)).unwrap();
        assert_eq!(2, *game.inventory().get(&Key::Ring(Ring::Void)).unwrap());
        assert_eq!(Some(Ring::Void), game.player.left_ring);
        assert!(game.player.right_ring.is_none());

        game.use_item(Key::Ring(Ring::Void)).unwrap();
        assert_eq!(1, *game.inventory().get(&Key::Ring(Ring::Void)).unwrap());
        assert_eq!(Some(Ring::Void), game.player.left_ring);
        assert_eq!(Some(Ring::Void), game.player.right_ring);

        game.use_item(Key::Ring(Ring::Void)).unwrap();
        assert_eq!(1, *game.inventory().get(&Key::Ring(Ring::Void)).unwrap());
        assert_eq!(Some(Ring::Void), game.player.left_ring);
        assert_eq!(Some(Ring::Void), game.player.right_ring);

        game.add_item(Box::new(Ring::Speed));
        game.use_item(Key::Ring(Ring::Speed)).unwrap();
        assert_eq!(2, *game.inventory().get(&Key::Ring(Ring::Void)).unwrap());
        assert_eq!(Some(Ring::Speed), game.player.left_ring);
        assert_eq!(Some(Ring::Void), game.player.right_ring);
    }

    #[test]
    fn test_ring_unequip() {
        let mut game = Game::new();

        game.add_item(Box::new(Ring::Void));
        game.add_item(Box::new(Ring::HP));
        game.use_item(Key::Ring(Ring::Void)).unwrap();
        assert!(game.inventory().get(&Key::Ring(Ring::Void)).is_none());
        assert_eq!(Some(Ring::Void), game.player.left_ring);

        game.use_item(Key::Ring(Ring::Void)).unwrap();
        assert!(game.inventory().get(&Key::Ring(Ring::Void)).is_some());
        assert!(game.player.left_ring.is_none());

        let base_hp = game.player.max_hp();
        game.use_item(Key::Ring(Ring::Void)).unwrap();
        game.use_item(Key::Ring(Ring::HP)).unwrap();
        assert!(game.inventory().get(&Key::Ring(Ring::Void)).is_none());
        assert!(game.inventory().get(&Key::Ring(Ring::HP)).is_none());
        assert_eq!(Some(Ring::HP), game.player.left_ring);
        assert_eq!(Some(Ring::Void), game.player.right_ring);
        assert!(game.player.max_hp() > base_hp);

        game.use_item(Key::Ring(Ring::HP)).unwrap();
        assert!(game.inventory().get(&Key::Ring(Ring::HP)).is_some());
        assert_eq!(Some(Ring::Void), game.player.left_ring);
        assert!(game.player.right_ring.is_none());
        assert_eq!(base_hp, game.player.max_hp());
    }

    #[test]
    fn test_run_ring() {
        let mut game = Game::new();
        assert!(game.maybe_spawn_enemy().is_some());

        game.player.equip_ring(Ring::Evade);
        assert!(game.maybe_spawn_enemy().is_none());

        game.player.equip_ring(Ring::Void);
        assert!(game.maybe_spawn_enemy().is_none());

        game.player.equip_ring(Ring::Void);
        assert!(game.maybe_spawn_enemy().is_some());
    }
}
