extern crate dirs;

use crate::character::Character;
use crate::item::Item;
use crate::location::Location;
use crate::log;
use crate::randomizer::random;
use crate::randomizer::Randomizer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs, io, path};
use tombstone::Tombstone;

pub mod battle;
pub mod tombstone;

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
    tombstones: HashMap<Location, Tombstone>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            location: Location::home(),
            player: Character::player(),
            gold: 0,
            inventory: HashMap::new(),
            tombstones: HashMap::new(),
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

    /// Remove the game data and reset this reference.
    /// Tombstones are preserved across games.
    pub fn reset(&mut self) {
        // move the tombstones to the new game
        let mut new_game = Self::new();
        new_game.tombstones = self.tombstones.drain().collect();

        // replace the current, finished game with the new one
        *self = new_game;
    }

    /// Move the hero's location towards the given destination, one directory
    /// at a time, with some chance of enemies appearing on each one.
    pub fn go_to(&mut self, dest: &Location, run: bool, bribe: bool) -> Result<(), Error> {
        while self.location != *dest {
            self.location.go_to(&dest);
            if self.location.is_home() {
                self.visit_home();
            } else if self.pick_up_tombstone() {
                return Ok(());
            } else if let Some(mut enemy) = self.maybe_spawn_enemy() {
                // don't attempt bribe and run in the same turn
                if bribe {
                    if self.bribe(&enemy) {
                        return Ok(());
                    }
                } else if run && self.run_away(&enemy) {
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

    /// If there's a tombstone laying in the current location, pick up its items
    fn pick_up_tombstone(&mut self) -> bool {
        if let Some(mut tombstone) = self.tombstones.remove(&self.location) {
            log::tombstone_found(&self.location);
            tombstone.pick_up(self);
            true
        } else {
            false
        }
    }

    fn maybe_spawn_enemy(&self) -> Option<Character> {
        let distance = self.location.distance_from_home();
        if random().should_enemy_appear(&distance) {
            let level = enemy_level(self.player.level, distance.len());
            let level = random().enemy_level(level);
            let enemy = Character::enemy(level, distance);
            log::enemy_appears(&enemy, &self.location);
            Some(enemy)
        } else {
            None
        }
    }

    fn bribe(&mut self, enemy: &Character) -> bool {
        let bribe_cost = gold_gained(enemy.level) / 2;

        if self.gold >= bribe_cost && random().bribe_succeeds() {
            self.gold -= bribe_cost;
            log::bribe_success(&self.player, bribe_cost);
            return true;
        };
        log::bribe_failure(&self.player);
        false
    }

    fn run_away(&self, enemy: &Character) -> bool {
        if random().run_away_succeeds(self.player.level, enemy.level) {
            log::run_away_success(&self.player);
            return true;
        };
        log::run_away_failure(&self.player);
        false
    }

    fn battle(&mut self, enemy: &mut Character) -> Result<(), Error> {
        if let Ok(xp) = battle::run(self, enemy, &random()) {
            let gold = gold_gained(enemy.level);
            self.gold += gold;
            let level_up = self.player.add_experience(xp);

            log::battle_won(&self.player, &self.location, xp, level_up, gold);
            Ok(())
        } else {
            // leave hero items in the location
            let tombstone = Tombstone::drop(self);
            self.tombstones.insert(self.location.clone(), tombstone);

            log::battle_lost(&self.player, &self.location);
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
    std::cmp::max(player_level / 2 + distance_from_home - 1, 1)
}

fn gold_gained(enemy_level: i32) -> i32 {
    random().gold_gained(enemy_level * 100)
}

#[cfg(test)]
mod tests {
    use crate::location::Distance;
    use item::equipment::Equipment;

    use super::*;
    use crate::item;
    use crate::randomizer;

    #[test]
    fn test_enemy_level() {
        // player level 1
        assert_eq!(1, enemy_level(1, 1));
        assert_eq!(1, enemy_level(1, 2));
        assert_eq!(2, enemy_level(1, 3));

        // Player level 5
        assert_eq!(2, enemy_level(5, 1));
        assert_eq!(3, enemy_level(5, 2));
        assert_eq!(4, enemy_level(5, 3));

        // player level 10
        assert_eq!(5, enemy_level(10, 1));
        assert_eq!(6, enemy_level(10, 2));
        assert_eq!(7, enemy_level(10, 3));
    }

    #[test]
    fn test_inventory() {
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

    // NOTE: this tests are random and brittle and therefore bad unit tests but they
    // give a reasonable measure of how difficult the game is, so they are better than
    // nothing
    // TODO should do the same with fixed character classes
    // e.g. verify all classes can be beat at the game
    #[test]
    fn test_not_unbeatable() {
        let times = 100;
        // The premise of this test is: a player with enough potions and its
        // level's equipment, should be able to beat any enemy of its same level
        // without relying in randomness.
        let (wins, lost_to) = run_battles_at(1, 1, times);
        assert_wins(times, wins, 0.75, &lost_to);

        let (wins, lost_to) = run_battles_at(1, 3, times);
        assert_wins(times, wins, 0.3, &lost_to);

        let (wins, lost_to) = run_battles_at(5, 5, times);
        assert_wins(times, wins, 0.75, &lost_to);

        let (wins, lost_to) = run_battles_at(10, 5, times);
        assert_wins(times, wins, 0.75, &lost_to);

        let (wins, lost_to) = run_battles_at(10, 10, times);
        assert_wins(times, wins, 0.4, &lost_to);

        let (wins, lost_to) = run_battles_at(15, 13, times);
        assert_wins(times, wins, 0.75, &lost_to);

        let (wins, lost_to) = run_battles_at(15, 15, times);
        assert_wins(times, wins, 0.5, &lost_to);

        // it shouldn't be too easy either --stronger enemies should have
        // chances of winning (even with all the equipment)
        let (wins, _) = run_battles_at(1, 6, times);
        assert_loses(times, wins, 0.2);

        let (wins, _) = run_battles_at(1, 10, times);
        assert_loses(times, wins, 0.3);

        let (wins, _) = run_battles_at(5, 10, times);
        assert_loses(times, wins, 0.2);

        let (wins, _) = run_battles_at(5, 15, times);
        assert_loses(times, wins, 0.35);

        let (wins, _) = run_battles_at(10, 15, times);
        assert_loses(times, wins, 0.15);

        let (wins, _) = run_battles_at(15, 20, times);
        assert_loses(times, wins, 0.15);
    }

    fn assert_wins(total: i32, wins: i32, expected_ratio: f64, lost_to: &Vec<String>) {
        assert!(
            wins as f64 >= total as f64 * expected_ratio,
            "won {} out of {}. Lost to {:?}",
            wins,
            total,
            lost_to
        );
    }

    fn assert_loses(total: i32, wins: i32, expected_ratio: f64) {
        let expected = (total as f64) * (1.0 - expected_ratio);
        assert!(
            (wins as f64) <= expected,
            "won {} out of {} expected at most {}",
            wins,
            total,
            expected
        );
    }

    fn run_battles_at(player_level: i32, distance: i32, times: i32) -> (i32, Vec<String>) {
        let mut wins = 0;
        let mut lost_to = Vec::new();

        // we don't want randomization turned off for this test
        let random = randomizer::DefaultRandomizer {};

        for _ in 0..times {
            let mut game = full_game_at(player_level);

            // duplicate randomization from the game
            let e_level = enemy_level(player_level, distance);
            let e_level = random.enemy_level(e_level);
            let mut enemy = Character::enemy(e_level, Distance::from(distance));

            if battle::run(&mut game, &mut enemy, &random).is_ok() {
                wins += 1
            } else {
                lost_to.push(format!("{}[{}]", enemy.name(), enemy.level));
            }
        }

        (wins, lost_to)
    }

    fn full_game_at(level: i32) -> Game {
        let mut game = Game::new();

        // get a player of the given level
        for _ in 0..level - 1 {
            game.player.add_experience(game.player.xp_for_next());
        }
        assert_eq!(level, game.player.level);

        // add potions of the given level
        for _ in 0..10 {
            game.add_item("potion", Box::new(item::Potion::new(level)));
        }

        // add equipment of the given level
        game.player.sword = Some(item::equipment::Sword::new(level));
        game.player.shield = Some(item::equipment::Shield::new(level));

        game
    }
}
