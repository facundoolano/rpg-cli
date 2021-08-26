use super::ring::Ring;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::fmt;
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug, EnumIter)]
#[serde(try_from = "String", into = "String")]
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
        let key = match name.to_lowercase().as_str() {
            "potion" | "p" => Key::Potion,
            "ether" | "e" => Key::Ether,
            "remedy" | "r" => Key::Remedy,
            "escape" | "es" => Key::Escape,
            "sword" | "sw" => Key::Sword,
            "shield" | "sh" => Key::Shield,
            "hp-stone" | "hp" => Key::HealthStone,
            "mp-stone" | "mp" => Key::MagicStone,
            "str-stone" | "str" | "strength" => Key::PowerStone,
            "spd-stone" | "spd" | "speed" => Key::SpeedStone,
            "lvl-stone" | "level" | "lv" | "lvl" => Key::LevelStone,
            "void-rng" | "void" => Key::Ring(Ring::Void),
            "att-rng" | "att-ring" | "att" | "attack" | "attack-ring" | "attack-rng" => {
                Key::Ring(Ring::Attack)
            }
            "def-rng" | "def-ring" | "def" | "deffense" | "deffense-ring" | "deffense-rng" => {
                Key::Ring(Ring::Deffense)
            }
            "spd-rng" | "spd-ring" | "speed-ring" | "speed-rng" => Key::Ring(Ring::Speed),
            "mag-rng" | "mag-ring" | "mag" | "magic-ring" | "magic-rng" => Key::Ring(Ring::Magic),
            "mp-rng" | "mp-ring" => Key::Ring(Ring::MP),
            "hp-rng" | "hp-ring" => Key::Ring(Ring::HP),
            "evade-rng" | "evade" | "evade-ring" => Key::Ring(Ring::Evade),
            key => bail!("item {} not found", key),
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

// these From impls together with the serde try_from/into config
// allow Key variants to be used as keys in JSON objects for serialization
impl From<String> for Key {
    fn from(key: String) -> Self {
        Key::from(&key).unwrap()
    }
}

impl From<Key> for String {
    fn from(key_str: Key) -> Self {
        key_str.to_string()
    }
}

// Needed to implement EnumIter for tests
impl Default for Ring {
    fn default() -> Self {
        Ring::Void
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn from_into() {
        // verify that all existing keys can be parsed from strings
        // otherwise deserialization wouldn't be possible
        for key in Key::iter() {
            if let Key::Ring(_) = key {
                for ring in Ring::iter() {
                    let ring_key = Key::Ring(ring);
                    let parsed = Key::from(String::from(ring_key.clone()).as_str()).unwrap();
                    assert_eq!(ring_key, parsed);
                }
            } else {
                let parsed = Key::from(String::from(key.clone()).as_str()).unwrap();
                assert_eq!(key, parsed);
            }
        }
    }
}
