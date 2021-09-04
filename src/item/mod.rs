use core::fmt;

use crate::character::class as character;
use crate::game;
use crate::location;
use crate::log;
use serde::{Deserialize, Serialize};

pub mod chest;
pub mod equipment;
pub mod key;
pub mod ring;
pub mod shop;
pub mod stone;

#[typetag::serde(tag = "type")]
pub trait Item: fmt::Display {
    fn apply(&mut self, game: &mut game::Game);
    fn key(&self) -> key::Key;
    fn describe(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Potion {
    level: i32,
}

impl Potion {
    pub fn new(level: i32) -> Self {
        Self { level }
    }

    fn restores(&self) -> i32 {
        character::Class::player_first().hp.at(self.level) / 2
    }
}

impl fmt::Display for Potion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "potion[{}]", self.level)
    }
}

#[typetag::serde]
impl Item for Potion {
    fn apply(&mut self, game: &mut game::Game) {
        let recovered = game.player.update_hp(self.restores()).unwrap();
        log::heal_item(&game.player, "potion", recovered, 0, false);
    }

    fn key(&self) -> key::Key {
        key::Key::Potion
    }

    fn describe(&self) -> String {
        format!("restores {}hp", self.restores())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Escape {}

impl Escape {
    pub fn new() -> Self {
        Self {}
    }
}

#[typetag::serde]
impl Item for Escape {
    fn apply(&mut self, game: &mut game::Game) {
        game.visit(location::Location::home()).unwrap_or_default();
    }

    fn key(&self) -> key::Key {
        key::Key::Escape
    }

    fn describe(&self) -> String {
        String::from("transports the player safely back home")
    }
}

impl fmt::Display for Escape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "escape")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Remedy {}

impl Remedy {
    pub fn new() -> Self {
        Self {}
    }
}

#[typetag::serde]
impl Item for Remedy {
    fn apply(&mut self, game: &mut game::Game) {
        let healed = game.player.status_effect.take().is_some();
        log::heal_item(&game.player, "remedy", 0, 0, healed);
    }

    fn key(&self) -> key::Key {
        key::Key::Remedy
    }

    fn describe(&self) -> String {
        String::from("removes status ailments")
    }
}

impl fmt::Display for Remedy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "remedy")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ether {
    level: i32,
}

impl Ether {
    pub fn new(level: i32) -> Self {
        Self { level }
    }
}

impl fmt::Display for Ether {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ether[{}]", self.level)
    }
}

#[typetag::serde]
impl Item for Ether {
    fn apply(&mut self, game: &mut game::Game) {
        let to_restore = game
            .player
            .class
            .mp
            .as_ref()
            .map_or(0, |mp| mp.at(self.level));
        let recovered_mp = game.player.update_mp(to_restore);

        log::heal_item(&game.player, "ether", 0, recovered_mp, false);
    }

    fn key(&self) -> key::Key {
        key::Key::Ether
    }

    fn describe(&self) -> String {
        format!("restores level {} amount mp", self.level)
    }
}
