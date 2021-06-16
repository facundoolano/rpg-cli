use std::{fs, io, path};

pub struct NotFound;

pub fn read() -> Result<Vec<u8>, NotFound> {
    fs::read(file()).map_err(|_| NotFound)
}

pub fn write(data: Vec<u8>) -> Result<(), io::Error> {
    let rpg_dir = rpg_dir();
    if !rpg_dir.exists() {
        fs::create_dir(&rpg_dir).unwrap();
    }
    fs::write(file(), &data)
}

pub fn remove() {
    let rpg_dir = rpg_dir();
    if rpg_dir.exists() {
        fs::remove_dir_all(&rpg_dir).unwrap();
    }
}

fn rpg_dir() -> path::PathBuf {
    dirs::home_dir().unwrap().join(".rpg")
}

fn file() -> path::PathBuf {
    rpg_dir().join("data")
}
