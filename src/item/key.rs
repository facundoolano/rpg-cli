use serde::{Deserialize, Serialize};
use super::ring::Ring;
use anyhow::{bail, Result};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
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
    pub fn from(name: &str) -> Result<Self> {
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
            _ => bail!("Item not found"),
        };
        Ok(key)
    }
}


impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Key::Potion => "potion",
            Key::Escape => "escape",
            Key::Remedy => "remedy",
            Key::Ether => "ether",
            Key::HealthStone => "hp-stone",
            Key::MagicStone => "mp-stone",
            Key::PowerStone => "str-stone",
            Key::SpeedStone => "spd-stone",
            Key::LevelStone => "lvl-stone",
            Key::Sword => "sword",
            Key::Shield => "shield",
            Key::Ring(Ring::Void) => "void-rng",
            Key::Ring(Ring::Attack) => "att-rng",
            Key::Ring(Ring::Deffense) => "def-rng",
            Key::Ring(Ring::Speed) => "spd-rng",
            Key::Ring(Ring::Magic) => "mag-rng",
            Key::Ring(Ring::MP) => "mp-rng",
            Key::Ring(Ring::HP) => "hp-rng",
            Key::Ring(Ring::Evade) => "evade-rng",
        };

        write!(f, "{}", name)
    }
}
