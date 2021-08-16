use super::Item;
use crate::game;
use serde::{Deserialize, Serialize};

/// Rings are a wearable item that produce arbitrary effects hooked in
/// different places of the game, e.g. increase a stat, double gold gained, etc.
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum Ring {
    Void,
    Attack,
    Deffense,
    Speed,
    Magic,
    MP,
    HP,
}

impl Ring {
    // TODO should this be to_string instead?
    // FIXME consider this key to be a standard item thing
    pub fn key(&self) -> &'static str {
        match self {
            Ring::Void => "void",
            Ring::Attack => "attack",
            Ring::Deffense => "deffense",
            Ring::Speed => "speed",
            Ring::Magic => "magic",
            Ring::MP => "mp",
            Ring::HP => "hp",
        }
    }

    /// TODO
    pub fn factor(&self) -> f64 {
        match self {
            Ring::Attack => 0.5,
            Ring::Deffense => 0.5,
            Ring::Speed => 0.5,
            Ring::Magic => 0.5,
            Ring::MP => 0.5,
            Ring::HP => 0.5,
            _ => 0.0,
        }
    }
}

#[typetag::serde]
impl Item for Ring {
    /// When the ring is used, equip in the player. If the player was already
    /// wearing two rings, move the second one back to the inventory.
    fn apply(&mut self, game: &mut game::Game) {
        if let Some(removed) = game.player.equip_ring(self.clone()) {
            game.add_item(removed.key(), Box::new(removed));
        }
    }
}

/// The character is allowed to hold two rings.
/// This struct provides a simplified interface to get the ring pair net
/// contribution to different aspects of the game, e.g. character stats.
#[derive(Serialize, Deserialize, Default)]
pub struct RingSet {
    pub left: Option<Ring>,
    pub right: Option<Ring>,
}

// TODO rename to RingSet? RingHolder? RingEquip
impl RingSet {
    pub fn new() -> Self {
        Self {
            left: None,
            right: None,
        }
    }

    pub fn attack(&self, strength: i32) -> i32 {
        self.apply(strength, Ring::Attack)
    }

    pub fn deffense(&self, strength: i32) -> i32 {
        self.apply(strength, Ring::Deffense)
    }

    pub fn speed(&self, strength: i32) -> i32 {
        self.apply(strength, Ring::Deffense)
    }

    pub fn magic(&self, base: i32) -> i32 {
        self.apply(base, Ring::Magic)
    }

    pub fn mp(&self, base: i32) -> i32 {
        self.apply(base, Ring::MP)
    }

    pub fn hp(&self, base: i32) -> i32 {
        self.apply(base, Ring::HP)
    }

    /// TODO
    fn apply(&self, base: i32, ring: Ring) -> i32 {
        let factor =
            |r: &Option<Ring>| r.as_ref().filter(|&l| *l == ring).map_or(0.0, Ring::factor);
        let factor = factor(&self.left) + factor(&self.right);
        (base as f64 * factor).round() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character;

    #[test]
    fn test_ring_equip() {
        let mut game = game::Game::new();

        assert!(game.player.rings.left.is_none());
        assert!(game.player.rings.right.is_none());

        game.add_item("void", Box::new(Ring::Void));
        game.add_item("void", Box::new(Ring::Void));
        game.add_item("void", Box::new(Ring::Void));
        assert_eq!(3, *game.inventory().get("void").unwrap());

        game.use_item("void").unwrap();
        assert_eq!(2, *game.inventory().get("void").unwrap());
        assert_eq!(
            "void",
            game.player.rings.left.as_ref().map_or("fail", Ring::key)
        );
        assert!(game.player.rings.right.is_none());

        game.use_item("void").unwrap();
        assert_eq!(1, *game.inventory().get("void").unwrap());
        assert_eq!(
            "void",
            game.player.rings.left.as_ref().map_or("fail", Ring::key)
        );
        assert_eq!(
            "void",
            game.player.rings.right.as_ref().map_or("fail", Ring::key)
        );

        game.use_item("void").unwrap();
        assert_eq!(1, *game.inventory().get("void").unwrap());
        assert_eq!(
            "void",
            game.player.rings.left.as_ref().map_or("fail", Ring::key)
        );
        assert_eq!(
            "void",
            game.player.rings.right.as_ref().map_or("fail", Ring::key)
        );

        game.add_item("speed", Box::new(Ring::Speed));
        game.use_item("speed").unwrap();
        assert_eq!(2, *game.inventory().get("void").unwrap());
        assert_eq!(
            "speed",
            game.player.rings.left.as_ref().map_or("fail", Ring::key)
        );
        assert_eq!(
            "void",
            game.player.rings.right.as_ref().map_or("fail", Ring::key)
        );
    }

    fn test_character() -> character::Character {
        let stat = character::class::Stat(10, 1);
        character::Character::new(
            character::class::Class {
                name: String::from("test"),
                hp: stat.clone(),
                mp: Some(stat.clone()),
                strength: stat.clone(),
                speed: stat.clone(),
                category: character::class::Category::Player,
                inflicts: None,
            },
            1,
        )
    }

    #[test]
    fn test_apply_factor() {
        let mut rings = RingSet::new();
        assert_eq!(0, rings.apply(10, Ring::HP));
        rings.left = Some(Ring::Void);
        rings.right = Some(Ring::Void);
        assert_eq!(0, rings.apply(10, Ring::HP));

        rings.left = Some(Ring::HP);
        assert_eq!(5, rings.apply(10, Ring::HP));

        rings.right = Some(Ring::HP);
        assert_eq!(10, rings.apply(10, Ring::HP));
    }

    #[test]
    fn test_hp_ring() {
        let mut char = test_character();
        assert_eq!(10, char.current_hp);
        assert_eq!(10, char.max_hp());

        char.equip_ring(Ring::HP);
        assert_eq!(15, char.max_hp());
        assert_eq!(15, char.current_hp);

        char.equip_ring(Ring::HP);
        assert_eq!(20, char.max_hp());
        assert_eq!(20, char.current_hp);

        // push out to unequip
        char.equip_ring(Ring::Void);
        assert_eq!(15, char.max_hp());
        assert_eq!(15, char.current_hp);

        char.equip_ring(Ring::Void);
        assert_eq!(10, char.max_hp());
        assert_eq!(10, char.current_hp);
    }

    #[test]
    fn test_mp_ring() {}

    #[test]
    fn test_attack_ring() {}

    #[test]
    fn test_deffense_ring() {}

    #[test]
    fn test_magic_ring() {}

    #[test]
    fn test_speed_ring() {}
}
