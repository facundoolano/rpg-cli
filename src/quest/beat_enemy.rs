use std::collections::HashSet;

use super::Quest;
use crate::character::class;
use crate::character::class::Class;
use crate::event::Event;
use serde::{Deserialize, Serialize};

pub fn of_class(category: class::Category, description: &str) -> Box<dyn Quest> {
    let to_beat = Class::names(category);
    let total = to_beat.len();
    Box::new(BeatEnemyClass {
        to_beat,
        total,
        description: description.to_string(),
    })
}

pub fn shadow() -> Box<dyn Quest> {
    let mut to_beat = HashSet::new();
    to_beat.insert(String::from("shadow"));

    Box::new(BeatEnemyClass {
        to_beat,
        total: 1,
        description: String::from("beat your own shadow"),
    })
}

pub fn dev() -> Box<dyn Quest> {
    let mut to_beat = HashSet::new();
    to_beat.insert(String::from("dev"));

    Box::new(BeatEnemyClass {
        to_beat,
        total: 1,
        description: String::from("beat the dev"),
    })
}

pub fn at_distance(distance: i32) -> Box<dyn Quest> {
    Box::new(BeatEnemyDistance { distance })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeatEnemyClass {
    to_beat: HashSet<String>,
    total: usize,
    description: String,
}

#[typetag::serde]
impl Quest for BeatEnemyClass {
    fn description(&self) -> String {
        let already_beat = self.total - self.to_beat.len();
        format!("{} {}/{}", self.description, already_beat, self.total)
    }

    fn handle(&mut self, event: &Event) -> bool {
        if let Event::BattleWon { enemy, .. } = event {
            self.to_beat.remove(&enemy.name());
        }
        self.to_beat.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeatEnemyDistance {
    distance: i32,
}

#[typetag::serde]
impl Quest for BeatEnemyDistance {
    fn description(&self) -> String {
        format!("defeat an enemy {} steps away from home", self.distance)
    }

    fn handle(&mut self, event: &Event) -> bool {
        if let Event::BattleWon { location, .. } = event {
            if location.distance_from_home().len() >= self.distance {
                return true;
            }
        }
        false
    }
}
