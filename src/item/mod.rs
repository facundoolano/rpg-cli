use core::fmt;

use crate::character::class as character;
use crate::event::Event;
use crate::game;
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
        let to_restore = character::Class::HERO.hp.at(self.level) / 2;
        let recovered = game.player.heal(to_restore);

        Event::emit(
            game,
            Event::Heal {
                item: Some("potion"),
                recovered,
                healed: false,
            },
        );
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Remedy {}

impl Remedy {
    pub fn new() -> Self {
        Self {}
    }
}

#[typetag::serde]
impl Item for Remedy {
    fn apply(&self, game: &mut game::Game) {
        let healed = game.player.maybe_remove_status_effect();
        Event::emit(
            game,
            Event::Heal {
                item: Some("remedy"),
                recovered: 0,
                healed,
            },
        );
    }
}

impl fmt::Display for Remedy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "remedy")
    }
}
