use super::Item;
use crate::event::Event;
use crate::game;
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
    fn apply(&self, game: &mut game::Game) {
        let inc = game.player.increase_hp();
        event(game, "hp", inc);
    }
}

#[typetag::serde]
impl Item for Magic {
    fn apply(&self, game: &mut game::Game) {
        let inc = game.player.increase_mp();
        event(game, "mp", inc);
    }
}

#[typetag::serde]
impl Item for Power {
    fn apply(&self, game: &mut game::Game) {
        let inc = game.player.increase_strength();
        event(game, "str", inc);
    }
}

#[typetag::serde]
impl Item for Speed {
    fn apply(&self, game: &mut game::Game) {
        let inc = game.player.increase_speed();
        event(game, "spd", inc);
    }
}

#[typetag::serde]
impl Item for Level {
    fn apply(&self, game: &mut game::Game) {
        game.player.increase_level();
        event(game, "level", 1);
        Event::emit(
            game,
            Event::LevelUp {
                count: 1,
                current: game.player.level,
                class: game.player.name(),
            },
        )
    }
}

fn event(game: &mut game::Game, stat: &'static str, increase: i32) {
    Event::emit(game, Event::StoneUsed { stat, increase });
}
