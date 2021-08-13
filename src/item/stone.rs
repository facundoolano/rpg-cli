use super::Item;
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
        game.player.increase_hp();
    }
}

#[typetag::serde]
impl Item for Magic {
    fn apply(&self, game: &mut game::Game) {
        game.player.increase_mp();
    }
}

#[typetag::serde]
impl Item for Power {
    fn apply(&self, game: &mut game::Game) {
        game.player.increase_strength();
    }
}

#[typetag::serde]
impl Item for Speed {
    fn apply(&self, game: &mut game::Game) {
        game.player.increase_speed();
    }
}

#[typetag::serde]
impl Item for Level {
    fn apply(&self, game: &mut game::Game) {
        game.player.increase_level();
    }
}
