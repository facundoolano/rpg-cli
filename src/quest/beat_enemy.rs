use std::collections::HashSet;

use super::Quest;
use crate::character::class::Class;
use crate::event::Event;
use serde::{Deserialize, Serialize};

pub fn of_class(classes: &[Class], description: &str) -> Box<dyn Quest> {
    let to_beat = classes.iter().map(|c| c.name.to_string()).collect();
    Box::new(BeatEnemyClass {
        to_beat,
        total: classes.len(),
        description: description.to_string(),
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
