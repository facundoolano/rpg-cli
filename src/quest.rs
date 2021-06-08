use core::fmt;
use crate::game;
use crate::log;
use crate::character::Character;
use serde::{Deserialize, Serialize};

#[typetag::serde(tag = "type")]
pub trait Quest {
    /// What to show in the TODO quests list
    fn description (&self) -> &str;

    /// Whether this quest should appear in the quest list
    fn is_visible(&self, _game: &game::Game) -> bool {
        true
    }

    /// Whether this quest should be listed as TODO or DONE
    fn is_done(&self) -> bool;

    /// The gold rewarded upon quest completion
    fn reward(&self) -> i32;

    // Event handlers.
    // By default do nothing since most will only need to override one.
    fn battle_won(&mut self, _enemy: &Character, _levels_up: i32) {}
    fn item_bought(&mut self, _name: &str) {}
    fn item_used(&mut self, _name: &str) {}
    fn tombstone(&mut self) {}
}

impl fmt::Display for dyn Quest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// TODO
pub fn setup (game: &mut game::Game) {
    game.quests_todo.push(Box::new(WinBattle{done:false}));
}

/// TODO
pub fn list(game: &game::Game) {
    let todo: Vec<&str> = game.quests_todo.iter().filter(|q| q.is_visible(&game)).map(|q| q.description()).collect();
    log::quest_list(&todo, game.quests_done.as_slice());
}

// EVENT HANDLING
// TODO this could be simplified by having each quest registering to individual events

// alias for quests passed dynamically
type QuestRef<'a> = &'a mut Box<dyn Quest>;

pub fn handle_battle_won(game: &mut game::Game, enemy: &Character, levels_up: i32) {
    let handler = |q: QuestRef| q.battle_won(&enemy, levels_up);
    handle(game, &handler);
}

pub fn handle_item_bought(game: &mut game::Game, name: &str) {
    let handler = |q: QuestRef| q.item_bought(name);
    handle(game, &handler);
}

pub fn handle_item_used(game: &mut game::Game, name: &str) {
    let handler = |q: QuestRef| q.item_used(name);
    handle(game, &handler);
}

pub fn handle_tombstone(game: &mut game::Game) {
    let handler = |q: QuestRef| q.tombstone();
    handle(game, &handler);
}

fn handle(game: &mut game::Game, handler: &dyn Fn(QuestRef)) {
    let mut still_todo = Vec::new();
    for mut quest in game.quests_todo.drain(..) {
        handler(&mut quest);

        if quest.is_done() {
            let reward = quest.reward();
            game.gold += reward;
            log::quest_done(reward);

            // the done is stored from newer to older
            game.quests_done.insert(0, quest.description().to_string());
        } else {
            still_todo.push(quest);
        }
    }
    game.quests_todo = still_todo;
}

// QUEST DEFINITIONS
// TODO consider moving these to other files

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WinBattle {
    done: bool
}

#[typetag::serde]
impl Quest for WinBattle {
    fn description (&self) -> &str {
        "Win a battle"
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn reward(&self) -> i32 {
        100
    }

    fn battle_won(&mut self, _enemy: &Character, _levels_up: i32) {
        self.done = true;
    }
}
