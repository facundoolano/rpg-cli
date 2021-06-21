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
pub enum Event<'a> {
    EnemyAppears {
        enemy: &'a Character,
    },
    Bribe {
        cost: i32,
    },
    RunAway {
        success: bool,
    },
    Heal {
        item: Option<String>,
        recovered: i32,
        healed: bool,
    },
    BattleWon {
        enemy: &'a Character,
        location: Location,
        xp: i32,
        levels_up: i32,
        gold: i32,
    },
    BattleLost,
    LevelUp {
        current: i32,
    },
    ItemBought {
        item: String,
    },
    ItemUsed {
        item: String,
    },
    ChestFound {
        items: &'a [String],
        gold: i32,
    },
    TombstoneFound {
        items: &'a [String],
        gold: i32,
    },
}

impl Event<'_> {
    pub fn emit(game: &mut game::Game, event: Event) {
        log::handle(game, &event);
        quest::handle(game, &event);
    }
}

// FIXME remove these

pub fn status_effect_damage(character: &Character, damage: i32) {
    log::status_effect_damage(character, damage);
}

pub fn attack(character: &Character, attack: &battle::AttackType, damage: i32) {
    log::attack(character, attack, damage);
}

pub fn potion(game: &game::Game, restored: i32) {
    log::heal_item(&game.player, "potion", restored, false);
}

pub fn remedy(game: &game::Game, healed: bool) {
    log::heal_item(&game.player, "remedy", 0, healed);
}
