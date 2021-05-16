extern crate dirs;

use crate::character::Character;
use crate::item::Item;
use crate::location::Location;
use crate::log;
use crate::randomizer::Randomizer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs, io, path};

#[derive(Debug)]
pub enum Error {
    GameOver,
    NoDataFile,
    ItemNotFound,
}

pub enum Attack {
    Regular(i32),
    Critical(i32),
    Miss,
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
        // this could be generalized to player vs enemy parties
        let (mut pl_accum, mut en_accum) = (0, 0);
        let mut xp = 0;

        while !enemy.is_dead() {
            pl_accum += self.player.speed;
            en_accum += enemy.speed;

            if pl_accum >= en_accum {
                if !self.autopotion(enemy) {
                    let new_xp = self.player_attack(enemy);
                    xp += new_xp;
                }
                pl_accum = -1;
            } else {
                self.enemy_attack(enemy);
                en_accum = -1;
            }

            if self.player.is_dead() {
                log::battle_lost(&self.player, &self.location);
                return Err(Error::GameOver);
            }
        }

        let gold = gold_gained(enemy.level);
        self.gold += gold;
        let level_up = self.player.add_experience(xp);
        log::battle_won(&self.player, &self.location, xp, level_up, gold);

        Ok(())
    }

    /// If the player is low on hp and has a potion available use it
    /// instead of attacking in the current turn.
    fn autopotion(&mut self, enemy: &Character) -> bool {
        if self.player.current_hp > self.player.max_hp / 3 {
            return false;
        }

        // If there's a good chance of winning the battle on the next attack,
        // don't use the potion.
        let potential_damage = self.player.damage(&enemy);
        if potential_damage >= enemy.current_hp {
            return false;
        }

        // FIXME this prints in the non battle format
        self.use_item("potion").is_ok()
    }

    fn player_attack(&self, enemy: &mut Character) -> i32 {
        let (damage, new_xp) = Self::attack(&self.player, enemy);
        log::player_attack(&enemy, damage);
        new_xp
    }

    fn enemy_attack(&mut self, enemy: &Character) {
        let (damage, _) = Self::attack(enemy, &mut self.player);
        log::enemy_attack(&self.player, damage);
    }

    /// Inflict damage from attacker to receiver, return the inflicted
    /// damage and the experience that will be gain if the battle is won
    fn attack(attacker: &Character, receiver: &mut Character) -> (Attack, i32) {
        if Randomizer::should_miss(attacker.speed, receiver.speed) {
            (Attack::Miss, 0)
        } else {
            let damage = attacker.damage(&receiver);
            let xp = attacker.xp_gained(&receiver, damage);

            if Randomizer::should_critical() {
                let damage = damage * 2;
                receiver.receive_damage(damage);
                (Attack::Critical(damage), xp)
            } else {
                receiver.receive_damage(damage);
                (Attack::Regular(damage), xp)
            }
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

// these are kind of integration tests, may be better to move them out
#[cfg(test)]
mod tests {
    use super::*;
    use crate::item;
    use crate::location::Distance;

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
    fn battle_won() {
        let mut game = Game::new();
        // same level as player
        let mut enemy = Character::enemy(1, Distance::Near(1));

        game.player.speed = 2;
        game.player.current_hp = 20;
        game.player.strength = 10; // each hit will take 10hp

        enemy.speed = 1;
        enemy.current_hp = 15;
        enemy.strength = 5;

        // expected turns
        // enemy - 10hp
        // player - 5 hp
        // enemy - 10hp

        let result = game.battle(&mut enemy);
        assert!(result.is_ok());
        assert_eq!(15, game.player.current_hp);
        assert_eq!(1, game.player.level);
        assert_eq!(20, game.player.xp);
        assert_eq!(100, game.gold);

        let mut enemy = Character::enemy(1, Distance::Near(1));
        enemy.speed = 1;
        enemy.current_hp = 15;
        enemy.strength = 5;

        // same turns, added xp increases level

        let result = game.battle(&mut enemy);
        assert!(result.is_ok());
        assert_eq!(2, game.player.level);
        assert_eq!(10, game.player.xp);
        assert_eq!(200, game.gold);
    }

    #[test]
    fn battle_lost() {
        let mut game = Game::new();
        let near = Distance::Near(1);
        let mut enemy = Character::enemy(10, near);
        let result = game.battle(&mut enemy);
        assert!(result.is_err());
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
