use core::fmt;

use crate::character::class as character;
use crate::game;
use crate::log;
use serde::{Deserialize, Serialize};

pub mod shop;

pub trait Equipment: fmt::Display {
    fn new(level: i32) -> Self;

    fn level(&self) -> i32;

    /// How many strength points get added to the player when
    /// the item is equipped.
    fn strength(&self) -> i32 {
        // get the base strength of the hero at this level
        let player_strength = character::Class::HERO.strength_at(self.level());

        // calculate the added strength as a function of the player strength
        (player_strength as f64 * 1.5).round() as i32
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sword {
    level: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Shield {
    level: i32,
}

impl fmt::Display for Sword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sword[{}]", self.level())
    }
}

impl Equipment for Sword {
    fn new(level: i32) -> Self {
        Self { level }
    }

    fn level(&self) -> i32 {
        self.level
    }
}

impl fmt::Display for Shield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shield[{}]", self.level())
    }
}

impl Equipment for Shield {
    fn new(level: i32) -> Self {
        Self { level }
    }

    fn level(&self) -> i32 {
        self.level
    }
}

// TODO separate this and equipment into individual modules
#[typetag::serde(tag = "type")]
pub trait Item {
    fn apply(&self, game: &mut game::Game);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Potion {
    level: i32,
}

impl Potion {
    fn new(level: i32) -> Self {
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
    fn new() -> Self {
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
