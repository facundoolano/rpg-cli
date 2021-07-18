use crate::character::class;
use crate::game;
use std::{fs, io, path};

pub struct NotFound;

pub fn load() -> Result<game::Game, NotFound> {
    let data: Vec<u8> = read(data_file())?;
    let game = serde_json::from_slice(&data).unwrap();
    Ok(game)
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

fn rpg_dir() -> path::PathBuf {
    dirs::home_dir().unwrap().join(".rpg")
}

fn data_file() -> path::PathBuf {
    rpg_dir().join("data")
}

fn classes_file() -> path::PathBuf {
    rpg_dir().join("classes.yaml")
}
