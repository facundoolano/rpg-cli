use crate::player::Player;
use serde::{Deserialize, Serialize};
use std::{fs, io};

const DATA_FILE: &str = "~/.rpg/game";

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub player: Player,
    pub location: String,
}

impl Game {
    pub fn load() -> Result<Self, io::Error> {
        match fs::read(DATA_FILE) {
            Ok(data) => {
                let game: Game = bincode::deserialize(&data).unwrap();
                Ok(game)
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::new()),
            Err(error) => Err(error),
        }
    }

    pub fn save(&self) {
        // TODO implement
    }

    pub fn new() -> Self {
        Self {
            location: String::from("~"),
            player: Player::new(),
        }
    }
}
