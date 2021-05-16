extern crate dirs;

use crate::character::Character;
use crate::item::Item;
use crate::location::Location;
use crate::log;
use crate::randomizer::Randomizer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs, io, path};

pub mod battle;

#[derive(Debug)]
pub enum Error {
    GameOver,
    NoDataFile,
    ItemNotFound,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub player: Character,
    pub location: Location,
    pub gold: i32,
    inventory: HashMap<String, Vec<Box<dyn Item>>>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            location: Location::home(),
            player: Character::player(),
            gold: 0,
            inventory: HashMap::new(),
        }
    }

    pub fn load() -> Result<Self, Error> {
        let data = fs::read(data_file()).or(Err(Error::NoDataFile))?;
        let game: Game = bincode::deserialize(&data).unwrap();
        Ok(game)
    }

    pub fn save(&self) -> Result<(), io::Error> {
        let rpg_dir = rpg_dir();
        if !rpg_dir.exists() {
            fs::create_dir(&rpg_dir).unwrap();
        }

        let data = bincode::serialize(&self).unwrap();
        fs::write(data_file(), &data)
    }

    /// Remove the game data and reset this reference
    pub fn reset(&mut self) {
        let rpg_dir = rpg_dir();
        if rpg_dir.exists() {
            fs::remove_dir_all(&rpg_dir).unwrap();
        }
        *self = Self::new()
    }

    /// Move the hero's location towards the given destination, one directory
    /// at a time, with some chance of enemies appearing on each one.
    pub fn go_to(&mut self, dest: &Location, run: bool, bribe: bool) -> Result<(), Error> {
        while self.location != *dest {
            self.location.go_to(&dest);
            if self.location.is_home() {
                self.visit_home();
            } else if let Some(mut enemy) = self.maybe_spawn_enemy() {
                if bribe && self.bribe(&enemy) {
                    return Ok(());
                }

                if run && self.run_away(&enemy) {
                    return Ok(());
                }

                return self.battle(&mut enemy);
            }
        }
        Ok(())
    }

    /// Set the current location to home, and apply related side-effects
    pub fn visit_home(&mut self) {
        self.location = Location::home();
        let recovered = self.player.heal_full();
        log::heal(&self.player, &self.location, recovered);
    }

    pub fn add_item(&mut self, name: &str, item: Box<dyn Item>) {
        let entry = self
            .inventory
            .entry(name.to_string())
            .or_insert_with(Vec::new);
        entry.push(item);
    }

    pub fn use_item(&mut self, name: &str) -> Result<(), Error> {
        let name = name.to_string();
        // get all items of that type and use one
        // if there are no remaining, drop the type from the inventory
        if let Some(mut items) = self.inventory.remove(&name) {
            if let Some(item) = items.pop() {
                item.apply(self);
            }

            if !items.is_empty() {
                self.inventory.insert(name, items);
            }

            Ok(())
        } else {
            Err(Error::ItemNotFound)
        }
    }

    pub fn inventory(&self) -> HashMap<&str, usize> {
        self.inventory
            .iter()
            .map(|(k, v)| (k.as_ref(), v.len()))
            .collect::<HashMap<&str, usize>>()
    }

    fn maybe_spawn_enemy(&self) -> Option<Character> {
        let distance = self.location.distance_from_home();
        if Randomizer::should_enemy_appear(&distance) {
            let level = enemy_level(self.player.level, distance.len());
            let enemy = Character::enemy(level, distance);
            log::enemy_appears(&enemy, &self.location);
            Some(enemy)
        } else {
            None
        }
    }

    fn bribe(&mut self, enemy: &Character) -> bool {
        let bribe_cost = gold_gained(enemy.level) / 2;

        if self.gold >= bribe_cost && Randomizer::bribe_succeeds() {
            self.gold -= bribe_cost;
            log::bribe_success(&self.player, bribe_cost);
            return true;
        };
        log::bribe_failure(&self.player);
        false
    }

    fn run_away(&self, enemy: &Character) -> bool {
        if Randomizer::run_away_succeeds(self.player.level, enemy.level) {
            log::run_away_success(&self.player);
            return true;
        };
        log::run_away_failure(&self.player);
        false
    }

    fn battle(&mut self, enemy: &mut Character) -> Result<(), Error> {
        if let Ok(xp) = battle::run(self, enemy) {
            let gold = gold_gained(enemy.level);
            self.gold += gold;
            let level_up = self.player.add_experience(xp);
            log::battle_won(&self.player, &self.location, xp, level_up, gold);

            Ok(())
        } else {
            Err(Error::GameOver)
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

fn rpg_dir() -> path::PathBuf {
    dirs::home_dir().unwrap().join(".rpg")
}

fn data_file() -> path::PathBuf {
    rpg_dir().join("data")
}

fn enemy_level(player_level: i32, distance_from_home: i32) -> i32 {
    let random_delta = Randomizer::enemy_delta();
    std::cmp::max(player_level / 2 + distance_from_home - 1 + random_delta, 1)
}

fn gold_gained(enemy_level: i32) -> i32 {
    Randomizer::gold_gained(enemy_level * 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item;

    #[test]
    fn test_enemy_level() {
        // player level 1
        assert_eq!(1, enemy_level(1, 1));
        assert_eq!(1, enemy_level(1, 2));
        assert_eq!(2, enemy_level(1, 3));

        // player level 5
        assert_eq!(2, enemy_level(5, 1));
        assert_eq!(3, enemy_level(5, 2));
        assert_eq!(4, enemy_level(5, 3));

        // player level 10
        assert_eq!(5, enemy_level(10, 1));
        assert_eq!(6, enemy_level(10, 2));
        assert_eq!(7, enemy_level(10, 3));
    }

    #[test]
    fn inventory() {
        let mut game = Game::new();

        assert_eq!(0, game.inventory().len());

        let potion = item::Potion::new(1);
        game.add_item("potion", Box::new(potion));
        assert_eq!(1, game.inventory().len());
        assert_eq!(1, *game.inventory().get("potion").unwrap());

        let potion = item::Potion::new(1);
        game.add_item("potion", Box::new(potion));
        assert_eq!(1, game.inventory().len());
        assert_eq!(2, *game.inventory().get("potion").unwrap());

        game.player.current_hp -= 3;
        assert_ne!(game.player.max_hp, game.player.current_hp);

        assert!(game.use_item("potion").is_ok());

        // check it actually restores the hp
        assert_eq!(game.player.max_hp, game.player.current_hp);

        // check item was consumed
        assert_eq!(1, game.inventory().len());
        assert_eq!(1, *game.inventory().get("potion").unwrap());

        assert!(game.use_item("potion").is_ok());
        assert_eq!(0, game.inventory().len());
        assert!(game.use_item("potion").is_err());
    }
}
