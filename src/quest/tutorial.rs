use super::{Event, Quest};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WinBattle {
    done: bool,
}

impl WinBattle {
    pub fn new() -> Self {
        Self { done: false }
    }
}

#[typetag::serde]
impl Quest for WinBattle {
    fn description(&self) -> &str {
        "Win a battle"
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn reward(&self) -> i32 {
        100
    }

    fn handle(&mut self, event: &Event) {
        if let Event::BattleWon { .. } = event {
            self.done = true;
        }
    }
}

// TODO add more tutorial quests
