use super::Quest;
use crate::event::Event;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReachLevel {
    target: i32,
}

impl ReachLevel {
    pub fn new(target: i32) -> Self {
        Self { target }
    }
}

#[typetag::serde]
impl Quest for ReachLevel {
    fn description(&self) -> String {
        format!("reach level {}", self.target)
    }

    fn handle(&mut self, event: &Event) -> bool {
        if let Event::LevelUp { current, .. } = event {
            return *current >= self.target;
        }
        false
    }
}

const TOTAL_LEVELS: i32 = 5;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RaiseClassLevels {
    remaining: i32,
    class_name: String,
}

#[typetag::serde]
impl Quest for RaiseClassLevels {
    fn description(&self) -> String {
        let progress = TOTAL_LEVELS - self.remaining;
        format!(
            "raise {} levels with class {} {}/{}",
            TOTAL_LEVELS, self.class_name, progress, TOTAL_LEVELS
        )
    }

    fn handle(&mut self, event: &Event) -> bool {
        if let Event::LevelUp { count, class, .. } = event {
            if *class == self.class_name {
                self.remaining -= count;
            }
        } else if let Event::GameReset = event {
            self.remaining = TOTAL_LEVELS
        }
        self.remaining <= 0
    }
}

impl RaiseClassLevels {
    pub fn new(class_name: &str) -> Self {
        Self {
            remaining: TOTAL_LEVELS,
            class_name: class_name.to_string(),
        }
    }
}
