use super::Item;
use crate::game;
use serde::{Deserialize, Serialize};

/// Rings are a wearable item that produce arbitrary effects hooked in
/// different places of the game, e.g. increase a stat, double gold gained, etc.
#[derive(Serialize, Deserialize, PartialEq, Clone)]
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

    /// Run away always succeeds when equipped
    Run,
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
            Ring::Run => "run",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character;

    #[test]
    fn test_ring_equip() {
        let mut game = game::Game::new();

        assert!(game.player.equip.left_ring.is_none());
        assert!(game.player.equip.right_ring.is_none());

        game.add_item("void", Box::new(Ring::Void));
        game.add_item("void", Box::new(Ring::Void));
        game.add_item("void", Box::new(Ring::Void));
        assert_eq!(3, *game.inventory().get("void").unwrap());

        game.use_item("void").unwrap();
        assert_eq!(2, *game.inventory().get("void").unwrap());
        assert_eq!(
            "void",
            game.player
                .equip
                .left_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );
        assert!(game.player.equip.right_ring.is_none());

        game.use_item("void").unwrap();
        assert_eq!(1, *game.inventory().get("void").unwrap());
        assert_eq!(
            "void",
            game.player
                .equip
                .left_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );
        assert_eq!(
            "void",
            game.player
                .equip
                .right_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );

        game.use_item("void").unwrap();
        assert_eq!(1, *game.inventory().get("void").unwrap());
        assert_eq!(
            "void",
            game.player
                .equip
                .left_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );
        assert_eq!(
            "void",
            game.player
                .equip
                .right_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );

        game.add_item("speed", Box::new(Ring::Speed));
        game.use_item("speed").unwrap();
        assert_eq!(2, *game.inventory().get("void").unwrap());
        assert_eq!(
            "speed",
            game.player
                .equip
                .left_ring
                .as_ref()
                .map_or("fail", Ring::key)
        );
        assert_eq!(
            "void",
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
        let mut char = test_character();
        assert!(!char.equip.run_away_succeeds());

        char.equip_ring(Ring::Run);
        assert!(char.equip.run_away_succeeds());

        char.equip_ring(Ring::Void);
        assert!(char.equip.run_away_succeeds());

        char.equip_ring(Ring::Void);
        assert!(!char.equip.run_away_succeeds());
    }
}
