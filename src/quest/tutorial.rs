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
    fn description(&self) -> String {
        "win a battle".to_string()
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuySword {
    done: bool,
}

impl BuySword {
    pub fn new() -> Self {
        Self { done: false }
    }
}

#[typetag::serde]
impl Quest for BuySword {
    fn description(&self) -> String {
        "buy a sword".to_string()
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn reward(&self) -> i32 {
        100
    }

    fn handle(&mut self, event: &Event) {
        if let Event::ItemBought { item } = event {
            if item == "sword" {
                self.done = true;
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsePotion {
    done: bool,
}

impl UsePotion {
    pub fn new() -> Self {
        Self { done: false }
    }
}

#[typetag::serde]
impl Quest for UsePotion {
    fn description(&self) -> String {
        "use a potion".to_string()
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn reward(&self) -> i32 {
        100
    }

    fn handle(&mut self, event: &Event) {
        if let Event::ItemUsed { item } = event {
            if item == "potion" {
                self.done = true;
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReachLevel {
    target: i32,
    current: i32,
}

impl ReachLevel {
    pub fn new(level: i32) -> Self {
        Self {
            target: level,
            current: 1,
        }
    }
}

#[typetag::serde]
impl Quest for ReachLevel {
    fn description(&self) -> String {
        format!("reach level {}", self.target)
    }

    fn is_done(&self) -> bool {
        self.target <= self.current
    }

    fn reward(&self) -> i32 {
        100
    }

    fn handle(&mut self, event: &Event) {
        if let Event::BattleWon { levels_up, .. } = event {
            self.current += levels_up
        }
    }
}
