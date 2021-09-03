extern crate dirs;

use crate::character;
use crate::character::enemy;
use crate::character::Character;
use crate::item::chest::Chest;
use crate::item::key::Key;
use crate::item::ring::Ring;
use crate::item::Item;
use crate::location::Location;
use crate::log;
use crate::quest;
use crate::quest::QuestList;
use crate::randomizer::random;
use crate::randomizer::Randomizer;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Carries all the game state that is saved between commands and exposes
/// the high-level interface for gameplay: moving across directories and
/// engaging in battles.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Game {
    pub player: Character,
    pub location: Location,
    pub gold: i32,

    /// Items currently carried and unequipped
    pub inventory: HashMap<Key, Vec<Box<dyn Item>>>,

    /// Locations where chest have already been looked for, and therefore
    /// can't be found again.
    inspected: HashSet<Location>,

    /// Chests left at the location where the player dies.
    pub tombstones: HashMap<String, Chest>,

    /// There's one instance of each type of ring in the game.
    /// This set starts with all rings and they are moved to the inventory as
    /// they are found in chests.
    pub ring_pool: HashSet<Ring>,

    pub quests: QuestList,
}

impl Game {
    pub fn new() -> Self {
        let quests = QuestList::new();

        // There's one instance of each ring exiting per game.
        // The diamond ring is the only one that's found in the shop
        // instead of chests
        let mut ring_pool = Ring::set();
        ring_pool.remove(&Ring::Diamond);

        Self {
            location: Location::home(),
            player: Character::player(),
            gold: 0,
            inventory: HashMap::new(),
            tombstones: HashMap::new(),
            inspected: HashSet::new(),
            quests,
            ring_pool,
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

        quest::game_reset(self);
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

            if let Some(mut enemy) = enemy::spawn(&self.location, &self.player) {
                return self.battle(&mut enemy, run, bribe);
            }
        }
        Ok(())
    }

    /// Set the hero's location to the one given, and apply related side effects.
    pub fn visit(&mut self, location: Location) -> Result<(), character::Dead> {
        self.location = location;
        if self.location.is_home() {
            let (recovered_hp, recovered_mp, healed) = self.player.restore();
            log::heal(
                &self.player,
                &self.location,
                recovered_hp,
                recovered_mp,
                healed,
            );
        }

        // In location is home, already healed of negative status
        self.player.apply_status_effects()
    }

    /// Look for chests and tombstones at the current location.
    /// Remembers previously visited locations for consistency.
    pub fn inspect(&mut self) {
        if let Some(mut chest) = self.tombstones.remove(&self.location.to_string()) {
            let (items, gold) = chest.pick_up(self);
            log::tombstone(&items, gold);
            quest::tombstone(self);
        }

        if !self.inspected.contains(&self.location) {
            self.inspected.insert(self.location.clone());
            if let Some(mut chest) = Chest::generate(self) {
                let (items, gold) = chest.pick_up(self);
                log::chest(&items, gold);
                quest::chest(self);
            }
        }
    }

    pub fn add_item(&mut self, item: Box<dyn Item>) {
        let key = item.key();
        let entry = self.inventory.entry(item.key()).or_insert_with(Vec::new);
        entry.push(item);
        quest::item_added(self, key);
    }

    pub fn use_item(&mut self, name: Key) -> Result<()> {
        // get all items of that type and use one
        // if there are no remaining, drop the type from the inventory
        if let Some(mut items) = self.inventory.remove(&name) {
            if let Some(mut item) = items.pop() {
                item.apply(self);
                quest::item_used(self, item.key());
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
            quest::item_used(self, ring.key());
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

    /// Attempt to bribe or run away according to the given options,
    /// and start a battle if that fails.
    pub fn battle(
        &mut self,
        enemy: &mut Character,
        run: bool,
        bribe: bool,
    ) -> Result<(), character::Dead> {
        // don't attempt bribe and run in the same turn
        if bribe {
            let bribe_cost = self.player.gold_gained(enemy.level) / 2;
            if self.gold >= bribe_cost && random().bribe_succeeds() {
                self.gold -= bribe_cost;
                log::bribe(&self.player, bribe_cost);
                return Ok(());
            };
            log::bribe(&self.player, 0);
        } else if run {
            let success = random().run_away_succeeds(
                self.player.level,
                enemy.level,
                self.player.speed(),
                enemy.speed(),
            );
            log::run_away(&self.player, success);
            if success {
                return Ok(());
            }
        }

        if let Ok(xp) = self.run_battle(enemy) {
            self.battle_won(enemy, xp);
            Ok(())
        } else {
            self.battle_lost();
            Err(character::Dead)
        }
    }

    /// Runs a turn-based combat between the game's player and the given enemy.
    /// The frequency of the turns is determined by the speed stat of each
    /// character.
    ///
    /// Some special abilities are enabled by the player's equipped rings:
    /// Double-beat, counter-attack and revive.
    ///
    /// Returns Ok(xp gained) if the player wins, or Err(()) if it loses.
    fn run_battle(&mut self, enemy: &mut Character) -> Result<i32, character::Dead> {
        // Player's using the revive ring can come back to life at most once per battle
        let mut already_revived = false;

        // These accumulators get increased based on the character's speed:
        // the faster will get more frequent turns.
        let (mut pl_accum, mut en_accum) = (0, 0);
        let mut xp = 0;

        while enemy.current_hp > 0 {
            pl_accum += self.player.speed();
            en_accum += enemy.speed();

            if pl_accum >= en_accum {
                // In some urgent circumstances, it's preferable to use the turn to
                // recover mp or hp than attacking
                if !self.autopotion(enemy) && !self.autoether(enemy) {
                    let (new_xp, _) = self.player.attack(enemy);
                    xp += new_xp;

                    self.player.maybe_double_beat(enemy);
                }

                // Status effects are applied after each turn. The player may die
                // during its own turn because of status ailment damage
                let died = self.player.apply_status_effects();
                already_revived = self.player.maybe_revive(died, already_revived)?;

                pl_accum = -1;
            } else {
                let (_, died) = enemy.attack(&mut self.player);
                already_revived = self.player.maybe_revive(died, already_revived)?;

                self.player.maybe_counter_attack(enemy);

                enemy.apply_status_effects().unwrap_or_default();

                en_accum = -1;
            }
        }

        Ok(xp)
    }

    fn battle_won(&mut self, enemy: &Character, xp: i32) {
        let gold = self.player.gold_gained(enemy.level);
        self.gold += gold;
        let levels_up = self.player.add_experience(xp);

        let reward_items =
            Chest::battle_loot(self).map_or(HashMap::new(), |mut chest| chest.pick_up(self).0);

        log::battle_won(self, xp, levels_up, gold, &reward_items);
        quest::battle_won(self, enemy, levels_up);
    }

    fn battle_lost(&mut self) {
        // Drop hero items in the location. If there was a previous tombstone
        // merge the contents of both chests
        let mut tombstone = Chest::drop(self);
        let location = self.location.to_string();
        if let Some(previous) = self.tombstones.remove(&location) {
            tombstone.extend(previous);
        }
        self.tombstones.insert(location, tombstone);

        log::battle_lost(&self.player);
    }

    /// If the player is low on hp and has a potion available use it
    /// instead of attacking in the current turn.
    fn autopotion(&mut self, enemy: &Character) -> bool {
        if self.player.current_hp > self.player.max_hp() / 3 {
            return false;
        }

        // If there's a good chance of winning the battle on the next attack,
        // don't use the potion.
        let (potential_damage, _) = self.player.damage(enemy);
        if potential_damage >= enemy.current_hp {
            return false;
        }

        self.use_item(Key::Potion).is_ok()
    }

    fn autoether(&mut self, enemy: &Character) -> bool {
        if !self.player.class.is_magic() || self.player.can_magic_attack() {
            return false;
        }

        // If there's a good chance of winning the battle on the next attack,
        // don't use the ether.
        let (potential_damage, _) = self.player.damage(enemy);
        if potential_damage >= enemy.current_hp {
            return false;
        }

        self.use_item(Key::Ether).is_ok()
    }

}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
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
}
