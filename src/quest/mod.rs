use crate::character::Character;
use crate::game;
use crate::log;
use core::fmt;
use serde::{Deserialize, Serialize};

mod tutorial;

/// Events that can trigger quest updates.
enum Event {
    BattleWon { enemy: String, levels_up: i32 },
    ItemBought { item: String },
    ItemUsed { item: String },
    TombstoneFound,
}

/// Keeps a TODO list of quests for the game.
#[derive(Serialize, Deserialize, Default)]
pub struct QuestList {
    todo: Vec<Box<dyn Quest>>,
    done: Vec<String>,
}

impl QuestList {
    pub fn new() -> Self {
        let mut quests = Self {
            todo: Vec::new(),
            done: Vec::new(),
        };

        quests.setup();
        quests
    }

    /// Load the quests for a new game
    fn setup(&mut self) {
        self.todo.push(Box::new(tutorial::WinBattle::new()));
        self.todo.push(Box::new(tutorial::BuySword::new()));
        self.todo.push(Box::new(tutorial::UsePotion::new()));
        self.todo.push(Box::new(tutorial::ReachLevel::new(2)));
    }

    /// Pass the event to each of the quests, moving the completed ones to DONE.
    /// The total gold reward is returned.
    fn handle(&mut self, event: Event) -> i32 {
        let mut still_todo = Vec::new();
        let mut total_reward = 0;

        for mut quest in self.todo.drain(..) {
            quest.handle(&event);

            if quest.is_done() {
                let reward = quest.reward();
                total_reward += reward;
                log::quest_done(reward);

                // the done is stored from newer to older
                self.done.insert(0, quest.description().to_string());
            } else {
                still_todo.push(quest);
            }
        }

        self.todo = still_todo;
        total_reward
    }

    pub fn list(&self, game: &game::Game) -> (Vec<String>, Vec<String>) {
        let todo = self
            .todo
            .iter()
            .filter(|q| q.is_visible(&game))
            .map(|q| q.description())
            .collect();

        (todo, self.done.clone())
    }
}

/// A task that is assigned to the player when certain conditions are met.
/// New quests should implement this trait and be added to QuestList.setup method.
#[typetag::serde(tag = "type")]
trait Quest {
    /// What to show in the TODO quests list
    fn description(&self) -> String;

    /// Whether this quest should appear in the quest list
    fn is_visible(&self, _game: &game::Game) -> bool {
        true
    }

    /// Whether this quest should be listed as TODO or DONE
    fn is_done(&self) -> bool;

    /// The gold rewarded upon quest completion
    // NOTE: we could consider more sophisticated rewards than just gold
    fn reward(&self) -> i32;

    fn handle(&mut self, event: &Event);
}

impl fmt::Display for dyn Quest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

pub fn handle_battle_won(game: &mut game::Game, enemy: &Character, levels_up: i32) {
    handle(
        game,
        Event::BattleWon {
            enemy: enemy.name(),
            levels_up,
        },
    );
}

pub fn handle_item_bought(game: &mut game::Game, item: &str) {
    handle(
        game,
        Event::ItemBought {
            item: item.to_string(),
        },
    );
}

pub fn handle_item_used(game: &mut game::Game, item: &str) {
    handle(
        game,
        Event::ItemUsed {
            item: item.to_string(),
        },
    );
}

pub fn handle_tombstone(game: &mut game::Game) {
    handle(game, Event::TombstoneFound);
}

fn handle(game: &mut game::Game, event: Event) {
    game.gold += game.quests.handle(event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_completed() {
        let mut game = game::Game::new();
        let fake_enemy = Character::player();

        let initial_quests = game.quests.todo.len();
        assert!(initial_quests > 0);
        assert_eq!(0, game.quests.done.len());

        // first quest is to win a battle
        handle_battle_won(&mut game, &fake_enemy, 0);
        assert_eq!(initial_quests - 1, game.quests.todo.len());
        assert_eq!(1, game.quests.done.len());

        game.gold = 10;
        game.reset();
        // verify that the reset did something
        assert_eq!(0, game.gold);

        // verify that quests are preserved
        assert_eq!(initial_quests - 1, game.quests.todo.len());
        assert_eq!(1, game.quests.done.len());
    }
}
