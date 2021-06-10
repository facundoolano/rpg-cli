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
#[derive(Serialize, Deserialize, Default)]
pub struct QuestList {
    todo: Vec<(i32, Box<dyn Quest>)>,
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
        // FIXME should reward be handled here as well?

        self.todo.push((1, Box::new(tutorial::WinBattle::new())));
        self.todo.push((1, Box::new(tutorial::BuySword::new())));
        self.todo.push((1, Box::new(tutorial::UsePotion::new())));
        self.todo.push((1, Box::new(tutorial::ReachLevel::new(2))));

        self.todo.push((2, Box::new(tutorial::FindChest::new())));
        self.todo.push((2, Box::new(tutorial::ReachLevel::new(5))));
        self.todo.push((
            2,
            beat_enemy::of_class(&character::class::COMMON, "beat all common creatures"),
        ));

        self.todo.push((5, Box::new(tutorial::VisitTomb::new())));
        self.todo.push((5, Box::new(tutorial::ReachLevel::new(10))));
        self.todo.push((
            5,
            beat_enemy::of_class(&character::class::RARE, "beat all rare creatures"),
        ));
        self.todo.push((5, beat_enemy::at_distance(10)));

        // level 10
        self.todo.push((
            10,
            beat_enemy::of_class(&character::class::LEGENDARY, "beat all common creatures"),
        ));
    }

    /// Pass the event to each of the quests, moving the completed ones to DONE.
    /// The total gold reward is returned.
    fn handle(&mut self, event: Event) -> i32 {
        let mut still_todo = Vec::new();
        let mut total_reward = 0;

        for (unlock_at, mut quest) in self.todo.drain(..) {
            quest.handle(&event);

            if quest.is_done() {
                let reward = quest.reward();
                total_reward += reward;
                log::quest_done(reward);

                // the done is stored from newer to older
                self.done.insert(0, quest.description().to_string());
            } else {
                still_todo.push((unlock_at, quest));
            }
        }

        self.todo = still_todo;
        total_reward
    }

    pub fn list(&self, game: &game::Game) -> (Vec<String>, Vec<String>) {
        let todo = self
            .todo
            .iter()
            .filter(|(level, _)| level >= &game.player.level)
            .map(|(_, q)| q.description())
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
