use crate::character::class;
use crate::event;
use crate::game;
use crate::log;
use core::fmt;
use serde::{Deserialize, Serialize};

mod beat_enemy;
mod level;
mod tutorial;

pub fn handle(game: &mut game::Game, event: &event::Event) {
    // it would be preferable to have quests decoupled from the game struct
    // but that makes event handling much more complicated
    game.gold += game.quests.handle(event);
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum Status {
    Locked(i32),
    Unlocked,
    Completed,
}

/// Keeps a TODO list of quests for the game.
/// Each quest is unlocked at a certain level and has completion reward.
#[derive(Serialize, Deserialize, Default)]
pub struct QuestList {
    quests: Vec<(Status, i32, Box<dyn Quest>)>,
}

impl QuestList {
    pub fn new() -> Self {
        let mut quests = Self { quests: Vec::new() };

        quests.setup();
        quests
    }

    /// Load the quests for a new game
    fn setup(&mut self) {
        self.quests
            .push((Status::Unlocked, 100, Box::new(tutorial::WinBattle)));
        self.quests
            .push((Status::Unlocked, 100, Box::new(tutorial::BuySword)));
        self.quests
            .push((Status::Unlocked, 100, Box::new(tutorial::UsePotion)));
        self.quests
            .push((Status::Unlocked, 100, Box::new(level::ReachLevel::new(2))));

        self.quests
            .push((Status::Locked(2), 200, Box::new(tutorial::FindChest)));
        self.quests
            .push((Status::Locked(2), 500, Box::new(level::ReachLevel::new(5))));
        self.quests.push((
            Status::Locked(2),
            1000,
            beat_enemy::of_class(class::Category::Common, "beat all common creatures"),
        ));

        self.quests
            .push((Status::Locked(5), 200, Box::new(tutorial::VisitTomb)));
        self.quests.push((
            Status::Locked(5),
            1000,
            Box::new(level::ReachLevel::new(10)),
        ));
        self.quests.push((
            Status::Locked(5),
            5000,
            beat_enemy::of_class(class::Category::Rare, "beat all rare creatures"),
        ));
        self.quests
            .push((Status::Locked(5), 1000, beat_enemy::at_distance(10)));

        self.quests.push((
            Status::Locked(10),
            10000,
            beat_enemy::of_class(class::Category::Legendary, "beat all legendary creatures"),
        ));

        self.quests.push((
            Status::Locked(10),
            10000,
            Box::new(level::ReachLevel::new(50)),
        ));

        for name in class::Class::names(class::Category::Player) {
            self.quests.push((
                Status::Locked(10),
                5000,
                Box::new(level::RaiseClassLevels::new(&name)),
            ));
        }

        self.quests
            .push((Status::Locked(15), 20000, beat_enemy::shadow()));
        self.quests
            .push((Status::Locked(15), 20000, beat_enemy::dev()));

        self.quests.push((
            Status::Locked(50),
            100000,
            Box::new(level::ReachLevel::new(100)),
        ));
    }

    /// Pass the event to each of the quests, moving the completed ones to DONE.
    /// The total gold reward is returned.
    fn handle(&mut self, event: &event::Event) -> i32 {
        self.unlock_quests(event);

        let mut total_reward = 0;

        for (status, reward, quest) in &mut self.quests {
            if let Status::Completed = status {
                continue;
            }

            let is_done = quest.handle(event);
            if is_done {
                total_reward += *reward;
                log::quest_done(*reward);
                *status = Status::Completed
            }
        }

        total_reward
    }

    /// If the event is a level up, unlock quests for that level.
    fn unlock_quests(&mut self, event: &event::Event) {
        if let event::Event::LevelUp { current } = event {
            for (status, _, _) in &mut self.quests {
                if let Status::Locked(level) = status {
                    if *level <= *current {
                        *status = Status::Unlocked;
                    }
                }
            }
        }
    }

    pub fn list(&self) -> Vec<(bool, String)> {
        let mut result = Vec::new();

        for (status, _, q) in &self.quests {
            match status {
                Status::Locked(_) => {}
                Status::Unlocked => result.push((false, q.description())),
                Status::Completed => result.push((true, q.description())),
            };
        }
        result
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
    fn handle(&mut self, event: &event::Event) -> bool;
}

impl fmt::Display for dyn Quest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::Character;

    #[test]
    fn test_quest_completed() {
        let mut game = game::Game::new();
        let fake_enemy = Character::player();

        let initial_quests = count_status(&game.quests, Status::Unlocked);
        assert!(initial_quests > 0);
        assert_eq!(0, count_status(&game.quests, Status::Completed));

        // first quest is to win a battle
        let location = game.location.clone();
        event::Event::emit(
            &mut game,
            event::Event::BattleWon {
                enemy: &fake_enemy,
                location,
                xp: 100,
                levels_up: 0,
                gold: 100,
                player_class: "warrior".to_string(),
                items: &[],
            },
        );
        assert_eq!(
            initial_quests - 1,
            count_status(&game.quests, Status::Unlocked)
        );
        assert_eq!(1, count_status(&game.quests, Status::Completed));

        game.gold = 10;
        game.reset();
        // verify that the reset did something
        assert_eq!(0, game.gold);

        // verify that quests are preserved
        assert_eq!(
            initial_quests - 1,
            count_status(&game.quests, Status::Unlocked)
        );
        assert_eq!(1, count_status(&game.quests, Status::Completed));
    }

    fn count_status(quests: &QuestList, status: Status) -> usize {
        quests
            .quests
            .iter()
            .filter(|(q_status, _, _)| *q_status == status)
            .count()
    }
}
