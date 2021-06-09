use super::{Event, Quest};
use crate::game;
use serde::{Deserialize, Serialize};

// TODO consider using a macro to reduce duplication across these structs

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
        if let Event::EnemyBeat { .. } = event {
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
            if item.contains("sword") {
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
    unlock_at: i32,
    current: i32,
}

impl ReachLevel {
    pub fn new(target: i32, unlock_at: i32) -> Self {
        Self {
            target,
            unlock_at,
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

    fn is_visible(&self, game: &game::Game) -> bool {
        game.player.level >= self.unlock_at
    }

    fn reward(&self) -> i32 {
        100
    }

    fn handle(&mut self, event: &Event) {
        if let Event::LevelUp { current, .. } = event {
            self.current = *current
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FindChest {
    done: bool,
}

impl FindChest {
    pub fn new() -> Self {
        Self { done: false }
    }
}

#[typetag::serde]
impl Quest for FindChest {
    fn description(&self) -> String {
        "find a chest".to_string()
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn is_visible(&self, game: &game::Game) -> bool {
        game.player.level >= 2
    }

    fn reward(&self) -> i32 {
        100
    }

    fn handle(&mut self, event: &Event) {
        if let Event::ChestFound = event {
            self.done = true;
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisitTomb {
    done: bool,
}

impl VisitTomb {
    pub fn new() -> Self {
        Self { done: false }
    }
}

#[typetag::serde]
impl Quest for VisitTomb {
    fn description(&self) -> String {
        "visit the tomb of a fallen hero".to_string()
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn is_visible(&self, game: &game::Game) -> bool {
        game.player.level >= 5
    }

    fn reward(&self) -> i32 {
        100
    }

    fn handle(&mut self, event: &Event) {
        if let Event::TombstoneFound = event {
            self.done = true;
        }
    }
}
