use super::{key, Item};
use crate::event::Event;
use crate::game;
use crate::log;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Health;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Magic;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Power;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Speed;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Level;

#[typetag::serde]
impl Item for Health {
    fn apply(&mut self, game: &mut game::Game) {
        let inc = game.player.raise_hp();
        log(game, "hp", inc);
    }

    fn key(&self) -> key::Key {
        key::Key::HealthStone
    }
}

#[typetag::serde]
impl Item for Magic {
    fn apply(&mut self, game: &mut game::Game) {
        let inc = game.player.raise_mp();
        log(game, "mp", inc);
    }

    fn key(&self) -> key::Key {
        key::Key::MagicStone
    }
}

#[typetag::serde]
impl Item for Power {
    fn apply(&mut self, game: &mut game::Game) {
        let inc = game.player.raise_strength();
        log(game, "str", inc);
    }

    fn key(&self) -> key::Key {
        key::Key::PowerStone
    }
}

#[typetag::serde]
impl Item for Speed {
    fn apply(&mut self, game: &mut game::Game) {
        let inc = game.player.raise_speed();
        log(game, "spd", inc);
    }

    fn key(&self) -> key::Key {
        key::Key::SpeedStone
    }
}

#[typetag::serde]
impl Item for Level {
    fn apply(&mut self, game: &mut game::Game) {
        game.player.raise_level();
        log(game, "level", 1);
        Event::emit(
            game,
            Event::LevelUp {
                count: 1,
                current: game.player.level,
                class: game.player.name(),
            },
        )
    }

    fn key(&self) -> key::Key {
        key::Key::LevelStone
    }
}

fn log(game: &mut game::Game, stat: &'static str, increase: i32) {
    log::stat_increase(&game.player, stat, increase);
}
