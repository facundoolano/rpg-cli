use crate::location;
use serde::{Deserialize, Serialize};

use rand::Rng;

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    pub name: String,

    pub level: i32,
    pub xp: i32,

    pub max_hp: i32,
    pub current_hp: i32,

    pub strength: i32,
    pub speed: i32,
    pub luck: i32,
}

impl Character {
    // we could have a Character trait and separate Player and Enemy structs
    // but there's barely any logic to justify that yet
    pub fn player() -> Self {
        Self::new("hero", 1)
    }

    /// Spawn a new enemy in the given location, with level based on the
    /// player's current level and the distance of the location from home.
    pub fn enemy(location: &location::Location, player: &Self) -> Self {
        let distance: i32 = location.distance_from_home();
        Self::new("enemy", enemy_level(player.level, distance))
    }

    pub fn new(name: &str, level: i32) -> Self {
        Self {
            name: String::from(name),
            level,
            xp: 0,
            // FIXME arrange the rest of these stats based on level
            max_hp: 20,
            current_hp: 20,
            strength: 10,
            speed: 5,
            luck: 3,
        }
    }

    pub fn heal(&mut self) {
        self.current_hp = self.max_hp;
    }
}

fn enemy_level(player_level: i32, distance_from_home: i32) -> i32 {
    let mut rng = rand::thread_rng();
    let random_delta = rng.gen_range(-1..2);
    std::cmp::max(player_level / 2 + distance_from_home - 1 + random_delta, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy() {
        // player level 1
        assert!((1..=1).contains(&enemy_level(1, 1)));
        assert!((1..=2).contains(&enemy_level(1, 2)));
        assert!((1..=3).contains(&enemy_level(1, 3)));

        // player level 5
        assert!((1..=3).contains(&enemy_level(5, 1)));
        assert!((2..=4).contains(&enemy_level(5, 2)));
        assert!((3..=5).contains(&enemy_level(5, 3)));

        // player level 10
        assert!((4..=6).contains(&enemy_level(10, 1)));
        assert!((5..=7).contains(&enemy_level(10, 2)));
        assert!((6..=8).contains(&enemy_level(10, 3)));
    }
}
