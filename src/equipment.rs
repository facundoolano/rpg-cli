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
        // TODO make this a character method?
        let class = &character::Class::HERO;
        let inc_rate = 1.0 + class.strength_rate;
        let player_strength = class.start_strength as f64 * inc_rate.powi(self.level);

        // calculate the added strength as a function of the player strength
        (player_strength * 1.5).round() as i32
    }
}
