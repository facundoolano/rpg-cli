use crate::character::Character;
use crate::game;
use crate::game::battle;
use crate::location::Location;
use crate::log;
use crate::quest;

/// This module implements basic event management.
/// It's static, the events are not subscribed at runtime, but
/// it serves the purpose of decoupling logging and the quest system
/// from the rest of the codebase.
// NOTE: for now not adding variants when the only action is log,
// since there's no benefit on adding enum handling to the logging module
pub enum Event {
    EnemyBeat { enemy: String, location: Location },
    LevelUp { current: i32 },
    ItemBought { item: String },
    ItemUsed { item: String },
    ChestFound,
    TombstoneFound,
}

pub fn enemy_appears(game: &game::Game, enemy: &Character) {
    log::enemy_appears(&enemy, &game.location);
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

pub fn heal(game: &game::Game, recovered: i32, healed: bool) {
    log::heal(&game.player, &game.location, recovered, healed);
}

pub fn tombstone(game: &mut game::Game, items: &[String], gold: i32) {
    quest::handle(game, Event::TombstoneFound);
    log::tombstone(&items, gold);
}

pub fn chest(game: &mut game::Game, gold: i32, items: &[String]) {
    quest::handle(game, Event::ChestFound);

    if gold > 0 {
        log::chest_gold(gold);
    }
    if !items.is_empty() {
        log::chest_item(items);
    }
}

pub fn quest_complete(reward: i32) {
    log::quest_done(reward);
}

pub fn bribe(game: &game::Game, cost: i32) {
    if cost > 0 {
        log::bribe_success(&game.player, cost);
    } else {
        log::bribe_failure(&game.player);
    }
}

pub fn run_away(game: &game::Game, success: bool) {
    if success {
        log::run_away_success(&game.player);
    } else {
        log::run_away_failure(&game.player);
    }
}

pub fn damage(character: &Character, attack: &battle::Attack) {
    log::damage(character, attack);
}

pub fn status_effect(character: &Character) {
    // FIXME merge this with the regular damage, handle status inside of log::damage
    log::status_effect(character);
}

pub fn potion(game: &game::Game, restored: i32) {
    log::potion(&game.player, restored);
}

pub fn remedy(game: &game::Game, healed: bool) {
    log::remedy(&game.player, healed);
}
