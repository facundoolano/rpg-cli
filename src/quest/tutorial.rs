use super::Quest;
use crate::event::Event;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WinBattle;

#[typetag::serde]
impl Quest for WinBattle {
    fn description(&self) -> String {
        "win a battle".to_string()
    }

    fn handle(&mut self, event: &Event) -> bool {
        matches!(event, Event::BattleWon { .. })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuySword;

#[typetag::serde]
impl Quest for BuySword {
    fn description(&self) -> String {
        "buy a sword".to_string()
    }

    fn handle(&mut self, event: &Event) -> bool {
        if let Event::ItemBought { item } = event {
            if item.contains("sword") {
                return true;
            }
        }
        false
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsePotion;

#[typetag::serde]
impl Quest for UsePotion {
    fn description(&self) -> String {
        "use a potion".to_string()
    }

    fn handle(&mut self, event: &Event) -> bool {
        if let Event::ItemUsed { item } = event {
            if item == "potion" {
                return true;
            }
        }
        false
    }
}

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
            return *current == self.target;
        }
        false
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FindChest;

#[typetag::serde]
impl Quest for FindChest {
    fn description(&self) -> String {
        "find a chest".to_string()
    }

    fn handle(&mut self, event: &Event) -> bool {
        matches!(event, Event::ChestFound)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisitTomb;

#[typetag::serde]
impl Quest for VisitTomb {
    fn description(&self) -> String {
        "visit the tomb of a fallen hero".to_string()
    }

    fn handle(&mut self, event: &Event) -> bool {
        matches!(event, Event::TombstoneFound)
    }
}
