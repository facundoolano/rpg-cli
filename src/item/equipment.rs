use core::fmt;

use super::ring::Ring;
use crate::character::class::Class;
use serde::{Deserialize, Serialize};

/// Packages together different equipment pieces that can be worn by characters
/// or found in chests.
/// Provides a unified interface for stat contributions to the base stats of a
/// characters, e.g. the increased attack contributed by a sword and the
/// deffense contributed by a shield.
#[derive(Serialize, Deserialize, Default)]
pub struct Equipment {
    pub sword: Option<Weapon>,
    pub shield: Option<Weapon>,
    pub left_ring: Option<Ring>,
    pub right_ring: Option<Ring>,
}

impl Equipment {
    pub fn new() -> Self {
        Self {
            sword: None,
            shield: None,
            left_ring: None,
            right_ring: None,
        }
    }

    /// Take the sword and/or shield from the given equipment if
    /// self has none or has one with lower level.
    /// Returns a tuple indicating whether there were (sword, shield) upgrades.
    pub fn upgrade(&mut self, other: &mut Self) -> (bool, bool) {
        (
            maybe_upgrade(&mut self.sword, &mut other.sword),
            maybe_upgrade(&mut self.shield, &mut other.shield),
        )
    }

    /// Set the given ring as the left ring.
    /// If there was a left ring already, put it in the right.
    /// If there was a right ring already, return it.
    pub fn put_ring(&mut self, ring: Ring) -> Option<Ring> {
        let removed = self.right_ring.take();
        self.right_ring = self.left_ring.take();
        self.left_ring = Some(ring);
        removed
    }

    pub fn attack(&self, strength: i32) -> i32 {
        self.sword.as_ref().map_or(0, |s| s.strength())
            + self.ring_contribution(strength, Ring::Attack)
    }

    pub fn deffense(&self, strength: i32) -> i32 {
        self.shield.as_ref().map_or(0, |s| s.strength())
            + self.ring_contribution(strength, Ring::Deffense)
    }

    pub fn speed(&self, base: i32) -> i32 {
        self.ring_contribution(base, Ring::Speed)
    }

    pub fn magic(&self, base: i32) -> i32 {
        self.ring_contribution(base, Ring::Magic)
    }

    pub fn mp(&self, base: i32) -> i32 {
        self.ring_contribution(base, Ring::MP)
    }

    pub fn hp(&self, base: i32) -> i32 {
        self.ring_contribution(base, Ring::HP)
    }

    pub fn enemies_evaded(&self) -> bool {
        self.left_ring == Some(Ring::Evade) || self.right_ring == Some(Ring::Evade)
    }

    fn ring_contribution(&self, base: i32, ring: Ring) -> i32 {
        let factor =
            |r: &Option<Ring>| r.as_ref().filter(|&l| *l == ring).map_or(0.0, Ring::factor);
        let factor = factor(&self.left_ring) + factor(&self.right_ring);
        (base as f64 * factor).round() as i32
    }
}

fn maybe_upgrade(current: &mut Option<Weapon>, other: &mut Option<Weapon>) -> bool {
    if let Some(shield) = other.take() {
        if shield.is_upgrade_from(current) {
            current.replace(shield);
            return true;
        }
    }
    false
}

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
