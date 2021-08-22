use core::fmt;

use crate::character::class::Class;
use serde::{Deserialize, Serialize};

/// Equipment piece with a strength contribution based on
/// a level. Used to generically represent swords and shields.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Weapon {
    Sword(i32),
    Shield(i32),
}

impl Weapon {
    pub fn level(&self) -> i32 {
        match self {
            Weapon::Sword(level) => *level,
            Weapon::Shield(level) => *level,
        }
    }

    /// How many strength points get added to the player when
    /// the item is equipped.
    pub fn strength(&self) -> i32 {
        // get the base strength of the hero at this level
        let player_strength = Class::player_first().strength.at(self.level());

        // calculate the added strength as a function of the player strength
        (player_strength as f64 * 0.5).round() as i32
    }

    /// Return true if the other weapon either is None or has lower level than this one.
    pub fn is_upgrade_from(&self, maybe_other: &Option<Self>) -> bool {
        if let Some(equip) = maybe_other {
            self.level() > equip.level()
        } else {
            true
        }
    }
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Weapon::Sword(_) => "sword",
            Weapon::Shield(_) => "shield",
        };
        write!(f, "{}[{}]", name, self.level())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_factor() {
        let mut equip = Equipment::new();
        assert_eq!(0, equip.ring_contribution(10, Ring::HP));
        equip.left_ring = Some(Ring::Void);
        equip.right_ring = Some(Ring::Void);
        assert_eq!(0, equip.ring_contribution(10, Ring::HP));

        equip.left_ring = Some(Ring::HP);
        assert_eq!(5, equip.ring_contribution(10, Ring::HP));

        equip.right_ring = Some(Ring::HP);
        assert_eq!(10, equip.ring_contribution(10, Ring::HP));
    }
}
