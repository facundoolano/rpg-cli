extern crate dirs;

use crate::character::Character;
use crate::location::Location;
use crate::log;
use crate::randomizer::Randomizer;
use serde::{Deserialize, Serialize};
use std::{fs, io, path};

#[derive(Debug)]
pub enum Error {
    GameOver,
    NoDataFile,
}

pub enum Attack {
    Regular(i32),
    Critical(i32),
    Miss,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub player: Character,
    pub location: Location,
    pub gold: i32,
}

// TODO factor out all dir/file management code
fn rpg_dir() -> path::PathBuf {
    dirs::home_dir().unwrap().join(".rpg")
}

fn data_file() -> path::PathBuf {
    rpg_dir().join("data")
}

impl Game {
    pub fn new() -> Self {
        Self {
            location: Location::home(),
            player: Character::player(),
            gold: 0,
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

    pub fn reset(&self) {
        let rpg_dir = rpg_dir();
        if rpg_dir.exists() {
            fs::remove_dir_all(&rpg_dir).unwrap();
        }
    }

    /// Move the hero's location towards the given destination, one directory
    /// at a time, with some chance of enemies appearing on each one.
    pub fn go_to(&mut self, dest: &Location) -> Result<(), Error> {
        while self.location != *dest {
            self.location.go_to(&dest);
            if self.location.is_home() {
                self.visit_home();
            } else if let Some(mut enemy) = self.maybe_spawn_enemy() {
                return self.battle(&mut enemy);
            }
        }
        Ok(())
    }

    /// Set the current location to home, and apply related side-effects
    pub fn visit_home(&mut self) {
        self.location = Location::home();
        let recovered = self.player.heal();
        log::heal(&self.player, &self.location, recovered);
    }

    fn maybe_spawn_enemy(&self) -> Option<Character> {
        if Randomizer::should_enemy_appear() {
            let distance = self.location.distance_from_home();
            let level = enemy_level(self.player.level, distance);
            let enemy = Character::enemy(level, distance);
            log::enemy_appears(&enemy, &self.location);
            Some(enemy)
        } else {
            None
        }
    }

    fn battle(&mut self, enemy: &mut Character) -> Result<(), Error> {
        // this could be generalized to player vs enemy parties
        let (mut pl_accum, mut en_accum) = (0, 0);
        let player = &mut self.player;
        let mut xp = 0;

        while !enemy.is_dead() {
            pl_accum += player.speed;
            en_accum += enemy.speed;

            if pl_accum >= en_accum {
                let (damage, new_xp) = Self::attack(player, enemy);
                xp += new_xp;

                log::player_attack(&enemy, damage);
                pl_accum = -1;
            } else {
                let (damage, _) = Self::attack(enemy, player);
                log::enemy_attack(&player, damage);
                en_accum = -1;
            }

            if player.is_dead() {
                log::battle_lost(&player, &self.location);
                return Err(Error::GameOver);
            }
        }

        let gold = Randomizer::gold_gained(enemy.level * 100);
        self.gold += gold;
        let level_up = player.add_experience(xp);
        log::battle_won(&player, &self.location, xp, level_up, gold);

        Ok(())
    }

    /// Inflict damage from attacker to receiver, return the inflicted
    /// damage and the experience that will be gain if the battle is won
    fn attack(attacker: &mut Character, receiver: &mut Character) -> (Attack, i32) {
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

fn enemy_level(player_level: i32, distance_from_home: i32) -> i32 {
    let random_delta = Randomizer::enemy_delta();
    std::cmp::max(player_level / 2 + distance_from_home - 1 + random_delta, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut enemy = Character::enemy(1, 1);

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

        let mut enemy = Character::enemy(1, 1);
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
        let mut enemy = Character::enemy(10, 1);
        let result = game.battle(&mut enemy);
        assert!(result.is_err());
    }
}
