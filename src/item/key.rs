use super::ring::Ring;
use anyhow::{bail, Result};

pub enum Key {
    Potion,
    Escape,
    Remedy,
    Ether,
    HealthStone,
    MagicStone,
    PowerStone,
    SpeedStone,
    LevelStone,
    Sword,
    Shield,
    Ring(Ring),
}

impl Key {
    fn from(name: &str) -> Result<Self> {
        // FIXME add the previously officual values
        let key = match name.to_lowercase().as_str() {
            "p" => Key::Potion,
            "e" => Key::Ether,
            "es" => Key::Escape,
            "sw" => Key::Sword,
            "sh" => Key::Shield,
            "hp" => Key::HealthStone,
            "mp" => Key::MagicStone,
            "str" | "strength" => Key::PowerStone,
            "spd" | "speed" => Key::SpeedStone,
            "level" | "lv" | "lvl" => Key::LevelStone,
            "void" => Key::Ring(Ring::Void),
            "att-ring" | "att" | "attack" | "attack-ring" | "attack-rng" => Key::Ring(Ring::Attack),
            "def-ring" | "def" | "deffense" | "deffense-ring" | "deffense-rng" => {
                Key::Ring(Ring::Deffense)
            }
            "spd-ring" | "speed-ring" | "speed-rng" => Key::Ring(Ring::Speed),
            "mag-ring" | "mag" | "magic-ring" | "magic-rng" => Key::Ring(Ring::Magic),
            "mp-ring" => Key::Ring(Ring::MP),
            "hp-ring" => Key::Ring(Ring::HP),
            "evade" | "evade-ring" => Key::Ring(Ring::Evade),
            n => bail!("Item not found"),
        };
        Ok(key)
    }
}
