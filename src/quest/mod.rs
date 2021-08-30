use crate::character::class;
use crate::character::Character;
use crate::game;
use crate::location::Location;
use crate::log;
use core::fmt;
use serde::{Deserialize, Serialize};

mod beat_enemy;
mod level;
mod tutorial;

pub fn battle_won(game: &mut game::Game, enemy: &Character, levels_up: i32) {
    handle(
        game,
        Event::BattleWon {
            enemy,
            location: game.location.clone(),
        },
    );

    if levels_up > 0 {
        level_up(game, levels_up);
    }
}

pub fn level_up(game: &mut game::Game, count: i32) {
    handle(
        game,
        Event::LevelUp {
            count,
            current: game.player.level,
            class: game.player.name(),
        },
    );
}

pub fn item_bought(game: &mut game::Game, item: String) {
    handle(game, Event::ItemBought { item });
}

pub fn item_used(game: &mut game::Game, item: String) {
    handle(game, Event::ItemUsed { item });
}

pub fn chest(game: &mut game::Game) {
    handle(game, Event::ChestFound);
}

pub fn tombstone(game: &mut game::Game) {
    handle(game, Event::TombtsoneFound);
}

pub fn game_reset(game: &mut game::Game) {
    handle(game, Event::GameReset);
}

pub enum Event<'a> {
    BattleWon {
        enemy: &'a Character,
        location: Location,
    },
    LevelUp {
        count: i32,
        current: i32,
        class: String,
    },
    ItemBought {
        item: String,
    },
    ItemUsed {
        item: String,
    },
    ChestFound,
    TombtsoneFound,
    GameReset,
}

fn handle(game: &mut game::Game, event: Event) {
    // it would be preferable to have quests decoupled from the game struct
    // but that makes event handling much more complicated
    game.gold += game.quests.handle(&event);
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
    fn handle(&mut self, event: &Event) -> i32 {
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
    fn unlock_quests(&mut self, event: &Event) {
        if let Event::LevelUp { current, .. } = event {
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
    fn handle(&mut self, event: &Event) -> bool;
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
    use crate::item;
    use crate::item::Item;
    use std::collections::HashMap;

    #[test]
    fn test_quest_status() {
        let mut quests = QuestList { quests: Vec::new() };
        quests
            .quests
            .push((Status::Unlocked, 10, Box::new(level::ReachLevel::new(2))));
        quests
            .quests
            .push((Status::Locked(2), 20, Box::new(level::ReachLevel::new(3))));
        quests
            .quests
            .push((Status::Locked(3), 30, Box::new(level::ReachLevel::new(4))));
        quests
            .quests
            .push((Status::Locked(4), 40, Box::new(level::ReachLevel::new(5))));

        assert_eq!(1, count_status(&quests, Status::Unlocked));
        assert_eq!(0, count_status(&quests, Status::Completed));

        let reward = quests.handle(&event::Event::LevelUp {
            count: 1,
            current: 2,
            class: "warrior".to_string(),
        });
        assert_eq!(1, count_status(&quests, Status::Unlocked));
        assert_eq!(1, count_status(&quests, Status::Completed));
        assert_eq!(10, reward);

        let reward = quests.handle(&event::Event::LevelUp {
            count: 2,
            current: 4,
            class: "warrior".to_string(),
        });
        assert_eq!(1, count_status(&quests, Status::Unlocked));
        assert_eq!(3, count_status(&quests, Status::Completed));
        assert_eq!(50, reward);
    }

    #[test]
    fn test_game_quests() {
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
                items: HashMap::new(),
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

        // verify that it doesn't reward twice
        let location = game.location.clone();
        event::Event::emit(
            &mut game,
            event::Event::BattleWon {
                enemy: &fake_enemy,
                location,
                xp: 100,
                levels_up: 0,
                gold: 100,
                items: HashMap::new(),
            },
        );
        assert_eq!(0, game.gold);
        assert_eq!(
            initial_quests - 1,
            count_status(&game.quests, Status::Unlocked)
        );
        assert_eq!(1, count_status(&game.quests, Status::Completed));
    }

    #[test]
    fn test_level_up() {
        let mut game = game::Game::new();
        game.quests.quests = vec![
            (Status::Unlocked, 10, Box::new(level::ReachLevel::new(2))),
            (Status::Unlocked, 10, Box::new(level::ReachLevel::new(3))),
        ];

        game.player.level = 2;
        event::Event::emit(
            &mut game,
            event::Event::LevelUp {
                count: 1,
                class: "warrior".to_string(),
                current: 2,
            },
        );

        assert_eq!(Status::Completed, game.quests.quests[0].0);
        assert_eq!(Status::Unlocked, game.quests.quests[1].0);

        let mut stone = item::stone::Level;
        stone.apply(&mut game);
        assert_eq!(Status::Completed, game.quests.quests[0].0);
        assert_eq!(Status::Completed, game.quests.quests[1].0);
    }

    fn count_status(quests: &QuestList, status: Status) -> usize {
        quests
            .quests
            .iter()
            .filter(|(q_status, _, _)| *q_status == status)
            .count()
    }
}
