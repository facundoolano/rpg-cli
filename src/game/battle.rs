use super::Game;
use crate::character::{Character, Dead, StatusEffect};
use crate::event;
use crate::randomizer::Randomizer;

/// Outcome of an attack attempt.
/// This affects primarily how the attack is displayed.
pub enum AttackType {
    Regular,
    Critical,
    Effect(StatusEffect),
    Miss,
}

/// Run a turn-based combat between the game's player and the given enemy.
/// Return Ok(xp gained) if the player wins, or Err(()) if it loses.
pub fn run(game: &mut Game, enemy: &mut Character, random: &dyn Randomizer) -> Result<i32, Dead> {
    // These accumulators get increased based on the characters speed:
    // the faster will get more frequent turns.
    // This could be generalized to player vs enemy parties
    let (mut pl_accum, mut en_accum) = (0, 0);
    let mut xp = 0;

    while !enemy.is_dead() {
        pl_accum += game.player.speed;
        en_accum += enemy.speed;

        if pl_accum >= en_accum {
            if !autopotion(game, enemy) {
                let new_xp = attack(&mut game.player, enemy, random)?;
                xp += new_xp;
            }
            pl_accum = -1;
        } else {
            attack(enemy, &mut game.player, random).unwrap_or_default();
            en_accum = -1;
        }
    }

    Ok(xp)
}

/// Inflict damage from attacker to receiver, return the inflicted
/// damage and the experience that will be gain if the battle is won
fn attack(
    attacker: &mut Character,
    receiver: &mut Character,
    random: &dyn Randomizer,
) -> Result<i32, Dead> {
    let (attack_type, damage, new_xp) = generate_attack(attacker, receiver, random);
    event::attack(receiver, &attack_type, damage);
    receiver.receive_damage(damage)?;

    attacker.receive_status_effect_damage()?;
    Ok(new_xp)
}

/// Return randomized attack parameters according to the character attributes.
fn generate_attack(
    attacker: &Character,
    receiver: &mut Character,
    random: &dyn Randomizer,
) -> (AttackType, i32, i32) {
    let damage = random.damage(attacker.damage(receiver));
    let xp = attacker.xp_gained(receiver, damage);
    let attack_type = random.attack_type(
        attacker.inflicted_status_effect(),
        attacker.speed,
        receiver.speed,
    );

    match attack_type {
        AttackType::Miss => (attack_type, 0, 0),
        AttackType::Regular => (attack_type, damage, xp),
        AttackType::Critical => (attack_type, damage * 2, xp),
        AttackType::Effect(_) => (attack_type, damage, xp),
    }
}

/// If the player is low on hp and has a potion available use it
/// instead of attacking in the current turn.
fn autopotion(game: &mut Game, enemy: &Character) -> bool {
    if game.player.current_hp > game.player.max_hp / 3 {
        return false;
    }

    // If there's a good chance of winning the battle on the next attack,
    // don't use the potion.
    let potential_damage = game.player.damage(enemy);
    if potential_damage >= enemy.current_hp {
        return false;
    }

    game.use_item("potion").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Distance;

    #[test]
    fn won() {
        let mut game = Game::new();
        // same level as player
        let mut enemy = Character::enemy(1, Distance::Near(1));

        game.player.speed = 2;
        game.player.current_hp = 20;
        game.player.strength = 10; // each hit will take 10hp

        enemy.speed = 1;
        enemy.current_hp = 15;
        enemy.strength = 5;

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

        let mut enemy = Character::enemy(1, Distance::Near(1));
        enemy.speed = 1;
        enemy.current_hp = 15;
        enemy.strength = 5;

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
        let near = Distance::Near(1);
        let mut enemy = Character::enemy(10, near);
        let result = game.battle(&mut enemy);
        assert!(result.is_err());
    }
}
