use std::collections::HashSet;

use super::{Event, Quest};
use crate::character::class::Class;
use crate::game;
use serde::{Deserialize, Serialize};

pub fn of_class(classes: &[Class], description: &str, unlock_at: i32) -> Box<dyn Quest> {
    let to_beat = classes.iter().map(|c| c.name.to_string()).collect();
    Box::new(BeatEnemyClass {
        to_beat,
        total: classes.len(),
        description: description.to_string(),
        unlock_at,
    })
}

pub fn at_distance(distance: i32) -> Box<dyn Quest> {
    Box::new(BeatEnemyDistance {
        distance,
        done: false,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeatEnemyClass {
    to_beat: HashSet<String>,
    total: usize,
    unlock_at: i32,
    description: String,
}

#[typetag::serde]
impl Quest for BeatEnemyClass {
    fn description(&self) -> String {
        let already_beat = self.total - self.to_beat.len();
        format!("{} {}/{}", self.description, already_beat, self.total)
    }

    fn is_done(&self) -> bool {
        self.to_beat.is_empty()
    }

    fn is_visible(&self, game: &game::Game) -> bool {
        game.player.level >= self.unlock_at
    }

    fn reward(&self) -> i32 {
        self.unlock_at * 1000
    }

    fn handle(&mut self, event: &Event) {
        if let Event::EnemyBeat { enemy, .. } = event {
            self.to_beat.remove(enemy);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeatEnemyDistance {
    distance: i32,
    done: bool,
}

#[typetag::serde]
impl Quest for BeatEnemyDistance {
    fn description(&self) -> String {
        format!("Defeat an enemy {} steps away from home", self.distance)
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn reward(&self) -> i32 {
        2000
    }

    fn is_visible(&self, game: &game::Game) -> bool {
        game.player.level >= 5
    }

    fn handle(&mut self, event: &Event) {
        if let Event::EnemyBeat { location, .. } = event {
            if location.distance_from_home().len() >= self.distance {
                self.done = true;
            }
        }
    }
}
