use std::collections::HashSet;

use super::{Event, Quest};
use crate::character::class::Class;
use crate::game;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeatEnemies {
    to_beat: HashSet<String>,
    total: usize,
    unlock_at: i32,
    description: String,
}

impl BeatEnemies {
    pub fn of_class(classes: &[Class], description: &str, unlock_at: i32) -> Self {
        let to_beat = classes.iter().map(|c| c.name.to_string()).collect();
        Self {
            to_beat,
            total: classes.len(),
            description: description.to_string(),
            unlock_at,
        }
    }
}

#[typetag::serde]
impl Quest for BeatEnemies {
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
        self.unlock_at * 5000
    }

    fn handle(&mut self, event: &Event) {
        if let Event::EnemyBeat { enemy } = event {
            self.to_beat.remove(enemy);
        }
    }
}
