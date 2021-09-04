use super::{key, Item};
use crate::game;
use crate::log;
use crate::quest;
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

    fn describe(&self) -> String {
        String::from("raises hp")
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

    fn describe(&self) -> String {
        String::from("raises mp")
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

    fn describe(&self) -> String {
        String::from("raises strength")
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

    fn describe(&self) -> String {
        String::from("raises speed")
    }
}

#[typetag::serde]
impl Item for Level {
    fn apply(&mut self, game: &mut game::Game) {
        game.player.raise_level();
        log(game, "level", 1);
        quest::level_up(game, 1);
    }

    fn key(&self) -> key::Key {
        key::Key::LevelStone
    }

    fn describe(&self) -> String {
        String::from("raises the player level")
    }
}

fn log(game: &mut game::Game, stat: &'static str, increase: i32) {
    log::stat_increase(&game.player, stat, increase);
}
