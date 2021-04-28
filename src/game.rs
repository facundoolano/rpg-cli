use colored::*;
extern crate dirs;

use crate::character::Character;
use crate::location::Location;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{fs, io, path};

#[derive(Debug)]
pub enum Error {
    GameOver,
    NoDataFile,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub player: Character,
    pub location: Location,
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

    // TODO document
    // TODO use a less awkward name for such a central function.
    // maybe just `move`
    pub fn walk_towards(&mut self, dest: &Location) -> Result<(), Error> {
        while self.location != *dest {
            self.location.walk_towards(&dest);
            // println!("move {}", self.location);

            if self.location.is_home() {
                self.player.heal();
            } else if let Some(mut enemy) = self.maybe_spawn_enemy() {
                return self.battle(&mut enemy);
            }
        }
        Ok(())
    }

    fn maybe_spawn_enemy(&self) -> Option<Character> {
        if self.should_enemy_appear() {
            let level = self.enemy_level();
            Some(Character::new("enemy", level))
        } else {
            None
        }
    }

    fn should_enemy_appear(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_ratio(1, 3)
    }

    fn enemy_level(&self) -> i32 {
        let distance: i32 = self.location.distance_from_home();
        let mut rng = rand::thread_rng();
        let random_delta = rng.gen_range(-1..2);
        std::cmp::max(self.player.level / 2 + distance - 1 + random_delta, 1)
    }

    fn battle(&mut self, enemy: &mut Character) -> Result<(), Error> {
        println!("{}: {} appeared!", self.location, enemy);

        // this could be generalized to player vs enemy parties
        let (mut pl_accum, mut en_accum) = (0, 0);
        let player = &mut self.player;
        let mut xp = 0;
        while !enemy.is_dead() {
            pl_accum += player.speed;
            en_accum += enemy.speed;

            if pl_accum >= en_accum {
                xp += Self::attack(player, enemy);
                pl_accum = -1;
            } else {
                Self::attack(enemy, player);
                en_accum = -1;
            }

            if player.is_dead() {
                println!("{}", "you die!\n".bold().red());
                return Err(Error::GameOver);
            }
        }

        print!("{}", "you win!".bold().green());
        print!(" +{}xp", xp);
        if self.player.add_experience(xp) {
            print!(" +lv:{}", self.player.level);
        }
        println!("\n");

        Ok(())
    }

    /// Inflict damage from attacker to receiver and return the
    fn attack(attacker: &mut Character, receiver: &mut Character) -> i32 {
        let damage = attacker.damage(&receiver);
        receiver.receive_damage(damage);
        println!("{: <7} {:3}hp", receiver.name, -damage);

        attacker.xp_gained(&receiver, damage)
    }
}
