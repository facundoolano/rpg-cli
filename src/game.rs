extern crate dirs;

use crate::character::Character;
use crate::location::Location;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{fs, io, path};

#[derive(Debug)]
pub enum Error {
    GameOver,
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

    pub fn load() -> Result<Self, &'static str> {
        let data = fs::read(data_file()).or(Err("Data file not readable"))?;
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

    // TODO document
    // TODO use a less awkward name for such a central function.
    // maybe just `move`
    pub fn walk_towards(&mut self, dest: &Location) -> Result<(), Error> {
        while self.location != *dest {
            self.location.walk_towards(&dest);
            println!("move {}", self.location);

            if self.location.is_home() {
                self.player.heal();
            } else if let Some(enemy) = self.maybe_spawn_enemy() {
                return self.battle(&enemy);
            }
        }
        Ok(())
    }

    pub fn maybe_spawn_enemy(&self) -> Option<Character> {
        if self.should_enemy_appear() {
            let level = self.enemy_level();
            Some(Character::new("enemy", level))
        } else {
            None
        }
    }

    pub fn should_enemy_appear(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_ratio(1, 3)
    }

    pub fn enemy_level(&self) -> i32 {
        let distance: i32 = self.location.distance_from_home();
        let mut rng = rand::thread_rng();
        let random_delta = rng.gen_range(-1..2);
        std::cmp::max(self.player.level / 2 + distance - 1 + random_delta, 1)
    }

    pub fn battle(&mut self, enemy: &Character) -> Result<(), Error> {
        // TODO
        println!("enemy lv:{} appeared!", enemy.level);
        Err(Error::GameOver)
    }
}
