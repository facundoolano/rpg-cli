use crate::character;
use crate::character::Character;
use crate::game;
use crate::item::key::Key;
use crate::location::Location;
use crate::log;
use crate::quest;
use std::collections::HashMap;

/// This module implements basic event management. It's static: the events are
/// not subscribed at runtime, but it serves the purpose of decoupling logging
/// and the quest system from the rest of the codebase.
// NOTE these are not exhaustive, and the only included what we already need.
// In particular, events that are only used for display kind of abuse the fact
// that we already get a game instance in the handler, so current location and
// player are omitted
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
    PlayerAttack {
        enemy: &'a Character,
        kind: character::AttackType,
        damage: i32,
        mp_cost: i32,
    },
    EnemyAttack {
        kind: character::AttackType,
        damage: i32,
        mp_cost: i32,
    },
    StatusEffect {
        hp: i32,
        mp: i32,
    },
    BattleWon {
        enemy: &'a Character,
        location: Location,
        xp: i32,
        levels_up: i32,
        gold: i32,
        items: HashMap<Key, i32>,
    },
    BattleLost,
    LevelUp {
        count: i32,
        current: i32,
        class: String,
    },
    StoneUsed {
        stat: &'static str,
        increase: i32,
    },
    Heal {
        item: Option<&'static str>,
        recovered_hp: i32,
        recovered_mp: i32,
        healed: bool,
    },
    ItemBought {
        item: String,
    },
    ItemUsed {
        item: String,
    },
    ChestFound {
        items: HashMap<Key, i32>,
        gold: i32,
        is_tombstone: bool,
    },
    ClassChanged {
        lost_xp: i32,
    },
    GameReset,
}

impl Event<'_> {
    pub fn emit(game: &mut game::Game, event: Event) {
        log::handle(game, &event);
        quest::handle(game, &event);
    }
}
