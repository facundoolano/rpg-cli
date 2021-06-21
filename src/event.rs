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
// NOTE these kind of abuse the fact that we already get a game instance in the
// handler, so current location and player are omitted
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
        kind: battle::AttackType,
        damage: i32,
    },
    EnemyAttack {
        kind: battle::AttackType,
        damage: i32,
    },
    StatusEffectDamage {
        damage: i32,
    },
    BattleWon {
        enemy: &'a Character,
        // FIXME shouldn't be necessary, keeping because quests don't get game reference
        location: Location,
        xp: i32,
        levels_up: i32,
        gold: i32,
    },
    BattleLost,
    LevelUp {
        current: i32,
    },
    Heal {
        item: Option<&'static str>,
        recovered: i32,
        healed: bool,
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
