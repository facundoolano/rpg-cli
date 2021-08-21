use super::Item;
use crate::game;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Rings are a wearable item that produce arbitrary effects hooked in
/// different places of the game, e.g. increase a stat, double gold gained, etc.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Ring {
    /// No-effect ring
    Void,

    /// Increases physical attack
    Attack,

    /// Increases deffense
    Deffense,

    /// Increases speed stat
    Speed,

    /// Increases magical attack
    Magic,

    /// Increases max MP
    MP,

    /// Increases max HP
    HP,

    /// Enemies don't appear while wearing this ring
    Evade,
}

impl Ring {
    pub fn set() -> HashSet<Ring> {
        Ring::iter().collect()
    }

    // FIXME consider this key to be a standard item thing
    pub fn key(&self) -> &'static str {
        match self {
            Ring::Void => "void-rng",
            Ring::Attack => "att-rng",
            Ring::Deffense => "def-rng",
            Ring::Speed => "spd-rng",
            Ring::Magic => "mag-rng",
            Ring::MP => "mp-rng",
            Ring::HP => "hp-rng",
            Ring::Evade => "evade-rng",
        }
    }

    /// For stat modifying stats, return the factor that should be
    /// applied to the base character stat.
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

impl fmt::Display for Ring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key())
    }
}

#[typetag::serde]
impl Item for Ring {
    /// When the ring is used, equip in the player. If the player was already
    /// wearing two rings, move the second one back to the inventory.
    fn apply(&mut self, game: &mut game::Game) {
        if let Some(removed) = game.player.equip_ring(self.clone()) {
            game.add_item(&removed.to_string(), Box::new(removed));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character;

    #[test]
    fn test_ring_equip() {
        let mut game = game::Game::new();

        assert!(game.player.equip.left_ring.is_none());
        assert!(game.player.equip.right_ring.is_none());

        game.add_item("void-rng", Box::new(Ring::Void));
        game.add_item("void-rng", Box::new(Ring::Void));
        game.add_item("void-rng", Box::new(Ring::Void));
        assert_eq!(3, *game.inventory().get("void-rng").unwrap());

        game.use_item("void-rng").unwrap();
        assert_eq!(2, *game.inventory().get("void-rng").unwrap());
        assert_eq!(
            "void-rng",
            game.player
                .equip
                .left_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );
        assert!(game.player.equip.right_ring.is_none());

        game.use_item("void-rng").unwrap();
        assert_eq!(1, *game.inventory().get("void-rng").unwrap());
        assert_eq!(
            "void-rng",
            game.player
                .equip
                .left_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );
        assert_eq!(
            "void-rng",
            game.player
                .equip
                .right_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );

        game.use_item("void-rng").unwrap();
        assert_eq!(1, *game.inventory().get("void-rng").unwrap());
        assert_eq!(
            "void-rng",
            game.player
                .equip
                .left_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );
        assert_eq!(
            "void-rng",
            game.player
                .equip
                .right_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );

        game.add_item("spd-rng", Box::new(Ring::Speed));
        game.use_item("spd-rng").unwrap();
        assert_eq!(2, *game.inventory().get("void-rng").unwrap());
        assert_eq!(
            "spd-rng",
            game.player
                .equip
                .left_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );
        assert_eq!(
            "void-rng",
            game.player
                .equip
                .right_ring
                .as_ref()
                .map_or("fail", Ring::key)
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

        // preserve taken damage
        char.current_hp -= 3;

        char.equip_ring(Ring::HP);
        assert_eq!(15, char.max_hp());
        assert_eq!(12, char.current_hp);

        char.equip_ring(Ring::Void);
        char.equip_ring(Ring::Void);
        assert_eq!(10, char.max_hp());
        assert_eq!(7, char.current_hp);
    }

    #[test]
    fn test_mp_ring() {
        let mut char = test_character();
        assert_eq!(10, char.current_mp);
        assert_eq!(10, char.max_mp());

        char.equip_ring(Ring::MP);
        assert_eq!(15, char.max_mp());
        assert_eq!(15, char.current_mp);

        char.equip_ring(Ring::MP);
        assert_eq!(20, char.max_mp());
        assert_eq!(20, char.current_mp);

        // push out to unequip
        char.equip_ring(Ring::Void);
        assert_eq!(15, char.max_mp());
        assert_eq!(15, char.current_mp);

        char.equip_ring(Ring::Void);
        assert_eq!(10, char.max_mp());
        assert_eq!(10, char.current_mp);

        // preserve taken damage
        char.current_mp -= 3;

        char.equip_ring(Ring::MP);
        assert_eq!(15, char.max_mp());
        assert_eq!(12, char.current_mp);

        char.equip_ring(Ring::Void);
        char.equip_ring(Ring::Void);
        assert_eq!(10, char.max_mp());
        assert_eq!(7, char.current_mp);
    }

    #[test]
    fn test_attack_ring() {
        let mut char = test_character();
        char.class.mp = None;
        assert_eq!(10, char.physical_attack());

        char.equip_ring(Ring::Attack);
        assert_eq!(15, char.physical_attack());
    }

    #[test]
    fn test_deffense_ring() {
        let mut char = test_character();
        assert_eq!(0, char.deffense());

        char.equip_ring(Ring::Deffense);
        assert_eq!(5, char.deffense());
    }

    #[test]
    fn test_magic_ring() {
        let mut char = test_character();
        assert_eq!(30, char.magic_attack());

        char.equip_ring(Ring::Magic);
        assert_eq!(45, char.magic_attack());
    }

    #[test]
    fn test_speed_ring() {
        let mut char = test_character();
        assert_eq!(10, char.speed());

        char.equip_ring(Ring::Speed);
        assert_eq!(15, char.speed());
    }

    #[test]
    fn test_run_ring() {
        let mut game = game::Game::new();
        assert!(game.maybe_spawn_enemy().is_some());

        game.player.equip_ring(Ring::Evade);
        assert!(game.maybe_spawn_enemy().is_none());

        game.player.equip_ring(Ring::Void);
        assert!(game.maybe_spawn_enemy().is_none());

        game.player.equip_ring(Ring::Void);
        assert!(game.maybe_spawn_enemy().is_some());
    }
}
