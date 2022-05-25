use crate::character::class;
use crate::game;
use anyhow::{bail, Result};
use std::{fs, io, path};

struct NotFound;

pub fn load() -> Result<Option<game::Game>> {
    match read(data_file()) {
        Err(NotFound) => Ok(None),
        Ok(data) => {
            if let Ok(game) = serde_json::from_slice(&data) {
                Ok(Some(game))
            } else {
                bail!("Invalid game data file. If it was generated with a previous version please run `reset --hard` to restart.");
            }
        }
    }
}

pub fn save(game: &game::Game) -> Result<(), io::Error> {
    let data = serde_json::to_vec(game).unwrap();
    write(data_file(), data)
}

pub fn remove() {
    let rpg_dir = rpg_dir();
    if rpg_dir.exists() {
        fs::remove_file(data_file()).unwrap();
    }
}

pub fn load_classes() {
    if let Ok(bytes) = read(classes_file()) {
        class::Class::load(&bytes)
    }
}

fn read(file: path::PathBuf) -> Result<Vec<u8>, NotFound> {
    fs::read(file).map_err(|_| NotFound)
}

fn write(file: path::PathBuf, data: Vec<u8>) -> Result<(), io::Error> {
    let rpg_dir = rpg_dir();
    if !rpg_dir.exists() {
        fs::create_dir(&rpg_dir).unwrap();
    }
    fs::write(file, &data)
}

pub fn rpg_dir() -> path::PathBuf {
    //Home is checked first because that was the default in a previous version
    let home_dir = dirs::home_dir().unwrap().join(".rpg");
    if home_dir.exists() {
        home_dir
    } else {
        dirs::data_dir().unwrap().join("rpg")
    }
}

fn data_file() -> path::PathBuf {
    rpg_dir().join("data")
}

fn classes_file() -> path::PathBuf {
    rpg_dir().join("classes.yaml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::key;
    use crate::item::ring;

    #[test]
    fn serialize_ring() {
        // rings have a compound enum variant Key::Ring(Ring::_)
        // that doesn't work with the default enum serialization setup
        // this verifies the try_from = String workaround
        let mut game = game::Game::new();
        game.add_item(Box::new(ring::Ring::Void));
        let data = serde_json::to_vec(&game).unwrap();
        let mut game: game::Game = serde_json::from_slice(&data).unwrap();
        assert!(game.use_item(key::Key::Ring(ring::Ring::Void)).is_ok());
    }
}
