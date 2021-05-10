use crate::character::class as character;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Equipment {
    pub level: i32,
}

impl Equipment {
    pub fn new(level: i32) -> Self {
        Self { level }
    }

    /// How many strength points get added to the player when
    /// the item is equipped.
    pub fn strength(&self) -> i32 {
        // get the base strength of the hero at this level
        let player_strength = character::Class::HERO.strength_at(self.level);

        // calculate the added strength as a function of the player strength
        (player_strength as f64 * 1.5).round() as i32
    }
}
