extern crate dirs;

use crate::character::Character;
use crate::location::Location;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{fs, io, path};

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

    pub fn walk_towards(&mut self, dest: &Location) {
        while self.location != *dest {
            self.location.walk_towards(&dest);
            println!("move {}", self.location);

            if self.location.is_home() {
                self.player.heal();
            } else if let Some(enemy) = self.maybe_spawn_enemy() {
                self.battle(&enemy);
                return;
            }
        }
    }

    pub fn maybe_spawn_enemy(&self) -> Option<Character> {
        let mut rng = rand::thread_rng();
        if rng.gen_ratio(1, 3) {
            Some(Character::enemy(&self.location, &self.player))
        } else {
            None
        }
    }

    pub fn battle(&mut self, enemy: &Character) {
        // TODO
        println!("enemy lv:{} appeared!", enemy.level);
    }
}
