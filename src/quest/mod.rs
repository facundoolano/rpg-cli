use crate::character;
use crate::character::Character;
use crate::game;
use crate::location::Location;
use crate::log;
use core::fmt;
use serde::{Deserialize, Serialize};

mod beat_enemy;
mod tutorial;

/// Events that can trigger quest updates.
pub enum Event {
    EnemyBeat { enemy: String, location: Location },
    LevelUp { current: i32 },
    ItemBought { item: String },
    ItemUsed { item: String },
    ChestFound,
    TombstoneFound,
}

/// Keeps a TODO list of quests for the game.
/// Each quest is unlocked at a certain level and has completion reward.
#[derive(Serialize, Deserialize, Default)]
pub struct QuestList {
    todo: Vec<(i32, i32, Box<dyn Quest>)>,
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
        self.todo.push((1, 100, Box::new(tutorial::WinBattle)));
        self.todo.push((1, 100, Box::new(tutorial::BuySword)));
        self.todo.push((1, 100, Box::new(tutorial::UsePotion)));
        self.todo
            .push((1, 100, Box::new(tutorial::ReachLevel::new(2))));

        self.todo.push((2, 200, Box::new(tutorial::FindChest)));
        self.todo
            .push((2, 500, Box::new(tutorial::ReachLevel::new(5))));
        self.todo.push((
            2,
            1000,
            beat_enemy::of_class(&character::class::COMMON, "beat all common creatures"),
        ));

        self.todo.push((5, 200, Box::new(tutorial::VisitTomb)));
        self.todo
            .push((5, 1000, Box::new(tutorial::ReachLevel::new(10))));
        self.todo.push((
            5,
            5000,
            beat_enemy::of_class(&character::class::RARE, "beat all rare creatures"),
        ));
        self.todo.push((5, 1000, beat_enemy::at_distance(10)));

        self.todo.push((
            10,
            10000,
            beat_enemy::of_class(&character::class::LEGENDARY, "beat all common creatures"),
        ));
    }

    /// Pass the event to each of the quests, moving the completed ones to DONE.
    /// The total gold reward is returned.
    fn handle(&mut self, event: Event) -> i32 {
        let mut still_todo = Vec::new();
        let mut total_reward = 0;

        for (unlock_at, reward, mut quest) in self.todo.drain(..) {
            let is_done = quest.handle(&event);

            if is_done {
                total_reward += reward;
                log::quest_done(reward);

                // the done is stored from newer to older
                self.done.insert(0, quest.description().to_string());
            } else {
                still_todo.push((unlock_at, reward, quest));
            }
        }

        self.todo = still_todo;
        total_reward
    }

    pub fn list(&self, game: &game::Game) -> (Vec<String>, Vec<String>) {
        let todo = self
            .todo
            .iter()
            .filter(|(level, _, _)| &game.player.level >= level)
            .map(|(_, _, q)| q.description())
            .collect();

        (todo, self.done.clone())
    }
}

/// A task that is assigned to the player when certain conditions are met.
/// New quests should implement this trait and be added to QuestList.setup method.
#[typetag::serde(tag = "type")]
pub trait Quest {
    /// What to show in the TODO quests list
    fn description(&self) -> String;

    /// Update the quest progress based on the given event and
    /// return whether the quest was finished.
    fn handle(&mut self, event: &Event) -> bool;
}

impl fmt::Display for dyn Quest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

pub fn handle_battle_won(game: &mut game::Game, enemy: &Character, levels_up: i32) {
    handle(
        game,
        Event::EnemyBeat {
            enemy: enemy.name(),
            location: game.location.clone(),
        },
    );
    if levels_up > 0 {
        handle(
            game,
            Event::LevelUp {
                current: game.player.level,
            },
        );
    }
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

pub fn handle_chest(game: &mut game::Game) {
    handle(game, Event::ChestFound);
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
