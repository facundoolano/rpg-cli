use super::Item;
use crate::character;
use crate::game;
use serde::{Deserialize, Serialize};

/// Rings are a kind of equipment that produce arbitrary effects hooked in
/// different places of the game, e.g. increase a stat, double gold gained, etc.
#[derive(Serialize, Deserialize, PartialEq)]
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
    fn key(&self) -> &'static str {
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
    fn factor(&self) -> f64 {
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

    /// TODO explain
    fn equip_side_effect(&self, character: &mut character::Character) {
        match self {
            Ring::HP => {
                character.current_hp += (self.factor() * character.max_hp() as f64) as i32;
            }
            Ring::MP => {
                character.current_mp += (self.factor() * character.max_mp() as f64) as i32;
            }
            _ => {}
        }
    }

    /// TODO explain
    fn unequip_side_effect(&self, character: &mut character::Character) {
        match self {
            Ring::HP => {
                let to_remove = (self.factor() * character.max_hp() as f64) as i32;
                character.current_hp = std::cmp::max(1, character.current_hp - to_remove);
            }
            Ring::MP => {
                let to_remove = (self.factor() * character.max_mp() as f64) as i32;
                character.current_mp = std::cmp::max(1, character.current_mp - to_remove);
            }
            _ => {}
        }
    }
}

/// The character is allowed to hold two rings.
/// The ring pair struct is used to hold the rings that the character is wearing,
/// handling the equipping and calculating the net combined effect of the two rings.
#[derive(Serialize, Deserialize, Default)]
pub struct RingPair {
    left: Option<Ring>,
    right: Option<Ring>,
}

// TODO consider removing/reducing this one
// rename to RingSet? RingHolder? RingEquip
impl RingPair {
    pub fn new() -> Self {
        Self {
            left: None,
            right: None,
        }
    }

    /// Put the given ring in the left, moving the left (if any) to the right
    /// and returning the right (if any)
    // TODO update comment
    fn equip(character: &mut character::Character, ring: Ring) -> Option<Ring> {
        // Remove the right ring and unapply its side-effects
        let old_right = if let Some(removed) = character.rings.right.take() {
            // FIXME this should live in this class
            removed.unequip_side_effect(character);
            Some(removed)
        } else {
            None
        };

        // put the new ring in left, pushing the previous one
        // FIXME this should live in this class
        ring.equip_side_effect(character);
        character.rings.right = character.rings.left.replace(ring);

        old_right
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

/// RingItem is a wrapper that lets the rings be added to the inventory and
/// used them to equip them.
#[derive(Serialize, Deserialize)]
pub struct RingItem {
    ring: Ring,
}

// FIXME impl this trait for enum? could that work
#[typetag::serde]
impl Item for RingItem {
    /// When the item is used, equip the inner ring in the player.
    /// If the player was already wearing two rings, move the second one back
    /// to the inventory.
    fn apply(&mut self, game: &mut game::Game) {
        // In order to move out the inner ring (without having to change it to an Option)
        // replace its memory with a throw away Void ring
        let ring = std::mem::replace(&mut self.ring, Ring::Void);
        if let Some(removed) = RingPair::equip(&mut game.player, ring) {
            let key = removed.key();
            game.add_item(&key, Box::new(RingItem { ring: removed }));
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_equip() {
        let mut game = game::Game::new();

        assert!(game.player.rings.left.is_none());
        assert!(game.player.rings.right.is_none());

        game.add_item("void", Box::new(RingItem{ring: Ring::Void}));
        game.add_item("void", Box::new(RingItem{ring: Ring::Void}));
        game.add_item("void", Box::new(RingItem{ring: Ring::Void}));
        assert_eq!(3, *game.inventory().get("void").unwrap());

        game.use_item("void").unwrap();
        assert_eq!(2, *game.inventory().get("void").unwrap());
        assert_eq!("void", game.player.rings.left.as_ref().map_or("fail", Ring::key));
        assert!(game.player.rings.right.is_none());

        game.use_item("void").unwrap();
        assert_eq!(1, *game.inventory().get("void").unwrap());
        assert_eq!("void", game.player.rings.left.as_ref().map_or("fail", Ring::key));
        assert_eq!("void", game.player.rings.right.as_ref().map_or("fail", Ring::key));

        game.use_item("void").unwrap();
        assert_eq!(1, *game.inventory().get("void").unwrap());
        assert_eq!("void", game.player.rings.left.as_ref().map_or("fail", Ring::key));
        assert_eq!("void", game.player.rings.right.as_ref().map_or("fail", Ring::key));

        game.add_item("speed", Box::new(RingItem{ring: Ring::Speed}));
        game.use_item("speed").unwrap();
        assert_eq!(2, *game.inventory().get("void").unwrap());
        assert_eq!("speed", game.player.rings.left.as_ref().map_or("fail", Ring::key));
        assert_eq!("void", game.player.rings.right.as_ref().map_or("fail", Ring::key));
    }

    #[test]
    fn test_apply_factor() {
        let mut rings = RingPair::new();
        assert_eq!(0, rings.apply(10, Ring::HP));
        rings.left = Some(Ring::Void);
        rings.right = Some(Ring::Void);
        assert_eq!(0, rings.apply(10, Ring::HP));

        rings.left = Some(Ring::HP);
        assert_eq!(5, rings.apply(10, Ring::HP));

        rings.right = Some(Ring::HP);
        assert_eq!(10, rings.apply(10, Ring::HP));
    }

    fn test_game() -> game::Game {
        let mut game = game::Game::new();
        let stat = character::class::Stat(10, 1);
        let player = character::Character::new(character::class::Class{
            name: String::from("test"),
            hp: stat.clone(),
            mp: Some(stat.clone()),
            strength: stat.clone(),
            speed: stat.clone(),
            category: character::class::Category::Player,
            inflicts: None,
        }, 1);
        game.player = player;
        game
    }

    // FIXME this weirdness to use means the abstractions are not that solid?
    fn equip(game: &mut game::Game, ring: Ring) {
        let key = ring.key();
        let item = Box::new(RingItem{ring});
        game.add_item(&key, item);
        game.use_item(&key).unwrap();
    }

    // FIXME this test logic that partially lives somewhere else
    #[test]
    fn test_hp_ring() {
        let mut game = test_game();
        assert_eq!(10, game.player.current_hp);
        assert_eq!(10, game.player.max_hp());

        equip(&mut game, Ring::HP);
        assert_eq!(15, game.player.max_hp());
        assert_eq!(15, game.player.current_hp);

        // push out to unequip
        equip(&mut game, Ring::Void);
        equip(&mut game, Ring::Void);
        assert_eq!(10, game.player.max_hp());
        assert_eq!(10, game.player.current_hp);

        // FIXME what if double equip (for some reason)?
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
