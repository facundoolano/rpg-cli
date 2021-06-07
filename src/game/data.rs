use std::{fs, io, path};

pub fn read() -> io::Result<Vec<u8>> {
    fs::read(data_file())
}

pub fn write(data: Vec<u8>) -> Result<(), io::Error> {
    let rpg_dir = rpg_dir();
    if !rpg_dir.exists() {
        fs::create_dir(&rpg_dir).unwrap();
    }
    fs::write(data_file(), &data)
}

pub fn remove() {
    let rpg_dir = rpg_dir();
    if !rpg_dir.exists() {
        fs::remove_dir_all(&rpg_dir).unwrap();
    }
}

fn rpg_dir() -> path::PathBuf {
    dirs::home_dir().unwrap().join(".rpg")
}

fn data_file() -> path::PathBuf {
    rpg_dir().join("data")
}
