use core::fmt;

use crate::character::class as character;
use crate::game;
use crate::location;
use crate::log;
use serde::{Deserialize, Serialize};

pub mod shop;

#[derive(Serialize, Deserialize, Debug)]
pub struct Equipment {
    name: String,
    pub level: i32,
}

impl Equipment {
    pub fn new(name: &str, level: i32) -> Self {
        Self {
            level,
            name: name.to_string(),
        }
    }

    /// How many strength points get added to the player when
    /// the item is equipped.
    pub fn strength(&self) -> i32 {
        // get the base strength of the hero at this level
        let player_strength = character::Class::HERO.strength_at(self.level);

        // calculate the added strength as a function of the player strength
        (player_strength as f64 * 1.5).round() as i32
    }
}

impl std::fmt::Display for Equipment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.name, self.level)
    }
}

pub trait Item {
    fn apply(&self, game: &mut game::Game);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Potion {
    level: i32,
}

impl Potion {
    fn new(level: i32) -> Self {
        Self { level }
    }
}

impl std::fmt::Display for Potion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "potion[{}]", self.level)
    }
}

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

pub struct Escape {}

impl Escape {
    fn new() -> Self {
        Self {}
    }
}

impl Item for Escape {
    fn apply(&self, game: &mut game::Game) {
        game.location = location::Location::home();
        // FIXME duplication, move to game
        let recovered = game.player.heal();
        log::heal(&game.player, &game.location, recovered);
    }
}

impl std::fmt::Display for Escape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "escape")
    }
}
