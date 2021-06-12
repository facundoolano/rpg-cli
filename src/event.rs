use crate::character::Character;
use crate::game;
use crate::location::Location;
use crate::quest;

// TODO quest complete event
pub enum Event {
    EnemyBeat { enemy: String, location: Location },
    LevelUp { current: i32 },
    ItemBought { item: String },
    ItemUsed { item: String },
    ChestFound,
    TombstoneFound,
}

pub fn battle_won(game: &mut game::Game, enemy: &Character, levels_up: i32) {
    quest::handle(
        game,
        Event::EnemyBeat {
            enemy: enemy.name(),
            location: game.location.clone(),
        },
    );
    if levels_up > 0 {
        quest::handle(
            game,
            Event::LevelUp {
                current: game.player.level,
            },
        );
    }
}

pub fn item_bought(game: &mut game::Game, item: &str) {
    quest::handle(
        game,
        Event::ItemBought {
            item: item.to_string(),
        },
    );
}

pub fn item_used(game: &mut game::Game, item: &str) {
    quest::handle(
        game,
        Event::ItemUsed {
            item: item.to_string(),
        },
    );
}

pub fn tombstone(game: &mut game::Game) {
    quest::handle(game, Event::TombstoneFound);
}

pub fn chest(game: &mut game::Game) {
    quest::handle(game, Event::ChestFound);
}
