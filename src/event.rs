/// This module implements basic event management.
/// It's static, the events are not subscribed at runtime, but
/// it serves the purpose of decoupling logging and the quest system
/// from the rest of the codebase.

use crate::log;
use crate::character::Character;
use crate::game;
use crate::location::Location;
use crate::quest;

pub enum Event {
    EnemyBeat { enemy: String, location: Location },
    LevelUp { current: i32 },
    ItemBought { item: String },
    ItemUsed { item: String },
    ChestFound,
    TombstoneFound,
    QuestComplete{ reward:i32 },
}

pub fn battle_won(game: &mut game::Game, enemy: &Character, xp: i32, levels_up: i32, gold: i32) {
    log::battle_won(&game, xp, levels_up, gold);

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

pub fn battle_lost(game: &game::Game) {
    log::battle_lost(&game.player);
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

pub fn tombstone(game: &mut game::Game, items: &[String], gold: i32) {
    quest::handle(game, Event::TombstoneFound);
    log::tombstone(&items, gold);
}

pub fn chest(game: &mut game::Game) {
    quest::handle(game, Event::ChestFound);
}

pub fn quest_complete(reward: i32) {
    log::quest_done(reward);
}
