use super::beat_enemy;
use super::{Event, Quest};
use crate::item::key::Key;
use crate::item::ring::Ring;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EquipRing;

#[typetag::serde]
impl Quest for EquipRing {
    fn description(&self) -> String {
        "equip a ring".to_string()
    }

    fn handle(&mut self, event: &Event) -> bool {
        if let Event::ItemUsed { item: Key::Ring(_) } = event {
            return true;
        }
        false
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FindAllRings {
    to_find: HashSet<Ring>,
}

impl FindAllRings {
    pub fn new() -> Self {
        Self {
            to_find: Ring::set(),
        }
    }
}

#[typetag::serde]
impl Quest for FindAllRings {
    fn description(&self) -> String {
        let total = Ring::set().len();
        let already_found = total - self.to_find.len();
        format!("find all rings {}/{}", already_found, total)
    }

    fn handle(&mut self, event: &Event) -> bool {
        if let Event::ItemAdded {
            item: Key::Ring(ring),
        } = event
        {
            self.to_find.remove(ring);
        }
        self.to_find.is_empty()
    }
}

pub fn gorthaur() -> Box<dyn Quest> {
    let mut to_beat = HashSet::new();
    to_beat.insert(String::from("gorthaur"));

    Box::new(beat_enemy::BeatEnemyClass {
        to_beat,
        total: 1,
        description: String::from("carry the ruling ring to the deeps to meet its maker"),
    })
}
