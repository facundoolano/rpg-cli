use super::Game;
use crate::character::{Character, Dead};
use crate::item::key::Key;
use crate::log;

/// Run a turn-based combat between the game's player and the given enemy.
/// Return Ok(xp gained) if the player wins, or Err(()) if it loses.
pub fn run(game: &mut Game, enemy: &mut Character) -> Result<i32, Dead> {
    // These accumulators get increased based on the characters speed:
    // the faster will get more frequent turns.
    // This could be generalized to player vs enemy parties
    let (mut pl_accum, mut en_accum) = (0, 0);
    let mut xp = 0;

    while enemy.current_hp > 0 {
        pl_accum += game.player.speed();
        en_accum += enemy.speed();

        if pl_accum >= en_accum {
            if !autopotion(game, enemy) && !autoether(game, enemy) {
                let (new_xp, _) = game.player.attack(enemy);
                xp += new_xp;
            }

            // FIXME remove this method, call directly. maybe extract to helper
            game.apply_status_effects()?;
            pl_accum = -1;
        } else {
            let (_, dead) = enemy.attack(&mut game.player);
            dead?;

            // some duplication with game mod
            let (hp, mp) = enemy.apply_status_effects().unwrap_or_default();
            log::status_effect(enemy, hp, mp);

            en_accum = -1;
        }
    }

    Ok(xp)
}

/// If the player is low on hp and has a potion available use it
/// instead of attacking in the current turn.
fn autopotion(game: &mut Game, enemy: &Character) -> bool {
    if game.player.current_hp > game.player.max_hp() / 3 {
        return false;
    }

    // If there's a good chance of winning the battle on the next attack,
    // don't use the potion.
    let (potential_damage, _) = game.player.damage(enemy);
    if potential_damage >= enemy.current_hp {
        return false;
    }

    game.use_item(Key::Potion).is_ok()
}

fn autoether(game: &mut Game, enemy: &Character) -> bool {
    if !game.player.class.is_magic() || game.player.can_magic_attack() {
        return false;
    }

    // If there's a good chance of winning the battle on the next attack,
    // don't use the ether.
    let (potential_damage, _) = game.player.damage(enemy);
    if potential_damage >= enemy.current_hp {
        return false;
    }

    game.use_item(Key::Ether).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character;
    use crate::character::class;

    #[test]
    fn won() {
        let enemy_base = class::Class::random(class::Category::Common);
        let enemy_class = class::Class {
            speed: class::Stat(1, 1),
            hp: class::Stat(15, 1),
            strength: class::Stat(5, 1),
            ..enemy_base.clone()
        };
        let mut enemy = character::Character::new(enemy_class.clone(), 1);

        let mut game = Game::new();
        let player_class = class::Class {
            speed: class::Stat(2, 1),
            hp: class::Stat(20, 1),
            strength: class::Stat(10, 1), // each hit will take 10hp
            ..game.player.class.clone()
        };
        game.player = character::Character::new(player_class, 1);

        // expected turns
        // enemy - 10hp
        // player - 5 hp
        // enemy - 10hp

        let result = game.battle(&mut enemy);
        assert!(result.is_ok());
        assert_eq!(15, game.player.current_hp);
        assert_eq!(1, game.player.level);
        assert_eq!(20, game.player.xp);

        // extra 100g for the completed quest
        assert_eq!(150, game.gold);

        let mut enemy = character::Character::new(enemy_class.clone(), 1);

        // same turns, added xp increases level

        let result = game.battle(&mut enemy);
        assert!(result.is_ok());
        assert_eq!(2, game.player.level);
        assert_eq!(10, game.player.xp);
        // extra 100g for level up quest
        assert_eq!(300, game.gold);
    }

    #[test]
    fn lost() {
        let mut game = Game::new();
        let enemy_class = class::Class::random(class::Category::Common);
        let mut enemy = character::Character::new(enemy_class.clone(), 10);
        let result = game.battle(&mut enemy);
        assert!(result.is_err());
    }

    #[test]
    fn magic_attacks() {
        let mut game = Game::new();
        let enemy_base = class::Class::random(class::Category::Common);
        let enemy_class = class::Class {
            speed: class::Stat(1, 1),
            hp: class::Stat(100, 1),
            strength: class::Stat(5, 1),
            ..enemy_base.clone()
        };
        let mut enemy = character::Character::new(enemy_class, 1);

        game.player.change_class("mage").unwrap_or_default();
        let player_class = class::Class {
            speed: class::Stat(2, 1),
            hp: class::Stat(20, 1),
            strength: class::Stat(10, 1), // each hit will take 10hp
            mp: Some(class::Stat(10, 1)),
            ..game.player.class.clone()
        };
        game.player = character::Character::new(player_class, 1);

        // mage -mp with enough mp
        player_attack(&mut game, &mut enemy);
        assert_eq!(7, game.player.current_mp);
        assert_eq!(70, enemy.current_hp);

        player_attack(&mut game, &mut enemy);
        player_attack(&mut game, &mut enemy);
        assert_eq!(1, game.player.current_mp);
        assert_eq!(10, enemy.current_hp);

        // mage -mp=0 without enough mp
        player_attack(&mut game, &mut enemy);
        assert_eq!(1, game.player.current_mp);
        assert_eq!(7, enemy.current_hp);
    }
}
