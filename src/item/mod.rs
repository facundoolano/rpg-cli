use core::fmt;

use crate::character::class as character;
use crate::game;
use crate::log;
use serde::{Deserialize, Serialize};

pub mod equipment;
pub mod shop;

#[typetag::serde(tag = "type")]
pub trait Item {
    fn apply(&self, game: &mut game::Game);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Potion {
    level: i32,
}

impl Potion {
    pub fn new(level: i32) -> Self {
        Self { level }
    }
}

impl fmt::Display for Potion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "potion[{}]", self.level)
    }
}

#[typetag::serde]
impl Item for Potion {
    fn apply(&self, game: &mut game::Game) {
        let (current, max) = (game.player.current_hp, game.player.max_hp);
        let to_restore = character::Class::HERO.hp_at(self.level) / 2;
        // FIXME adapt player.heal for this
        let restored = std::cmp::min(to_restore, max - current);
        game.player.current_hp += restored;
        log::heal(&game.player, &game.location, restored);
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
    fn apply(&self, game: &mut game::Game) {
        game.visit_home();
    }
}

impl fmt::Display for Escape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "escape")
    }
}
