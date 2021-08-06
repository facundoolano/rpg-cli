use super::Game;
use crate::character::{Character, Dead, StatusEffect};
use crate::event::Event;
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
            if !autopotion(game, enemy) && !autoether(game, enemy) {
                let new_xp = player_attack(game, enemy, random);
                xp += new_xp;
            }

            game.maybe_receive_status_damage()?;
            pl_accum = -1;
        } else {
            enemy_attack(game, enemy, random)?;
            // TODO enemy receive status effect
            en_accum = -1;
        }
    }

    Ok(xp)
}

/// Attack enemy, returning the gained experience
fn player_attack(game: &mut Game, enemy: &mut Character, random: &dyn Randomizer) -> i32 {
    let (attack_type, damage, mp_cost, new_xp) = generate_attack(&game.player, enemy, random);
    enemy.receive_damage(damage).unwrap_or_default();
    game.player.current_mp -= mp_cost;

    Event::emit(
        game,
        Event::PlayerAttack {
            enemy,
            kind: attack_type,
            damage,
            mp_cost,
        },
    );
    new_xp
}

/// Attack player, returning Err(Dead) if the player dies.
fn enemy_attack(
    game: &mut Game,
    enemy: &mut Character,
    random: &dyn Randomizer,
) -> Result<(), Dead> {
    let (attack_type, damage, mp_cost, _xp) = generate_attack(enemy, &game.player, random);
    let result = game.player.receive_damage(damage);
    enemy.current_mp -= mp_cost;

    if let AttackType::Effect(status) = attack_type {
        game.player.status_effect = Some(status);
    }

    Event::emit(
        game,
        Event::EnemyAttack {
            kind: attack_type,
            damage,
            mp_cost,
        },
    );
    result
}

/// Return randomized attack parameters according to the character attributes.
fn generate_attack(
    attacker: &Character,
    receiver: &Character,
    random: &dyn Randomizer,
) -> (AttackType, i32, i32, i32) {
    let (damage, mp_cost) = attacker.damage(receiver);
    let damage = random.damage(damage);
    let xp = attacker.xp_gained(receiver, damage);

    let attack_type = random.attack_type(
        attacker.inflicted_status_effect(),
        attacker.speed,
        receiver.speed,
    );

    match attack_type {
        AttackType::Miss => (attack_type, 0, mp_cost, 0),
        AttackType::Regular => (AttackType::Regular, damage, mp_cost, xp),
        AttackType::Critical => (attack_type, damage * 2, mp_cost, xp),
        AttackType::Effect(status) if Some(status) != receiver.status_effect => {
            (attack_type, damage, mp_cost, xp)
        }
        // don't double-inflict if already has the same status
        AttackType::Effect(_) => (AttackType::Regular, damage, mp_cost, xp),
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
    let (potential_damage, _mp_cost) = game.player.damage(enemy);
    if potential_damage >= enemy.current_hp {
        return false;
    }

    game.use_item("potion").is_ok()
}

fn autoether(game: &mut Game, enemy: &Character) -> bool {
    if !game.player.class.is_magic() || game.player.can_magic_attack() {
        return false;
    }

    // If there's a good chance of winning the battle on the next attack,
    // don't use the ether.
    let (potential_damage, _mp_cost) = game.player.damage(enemy);
    if potential_damage >= enemy.current_hp {
        return false;
    }

    game.use_item("ether").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character;
    use crate::character::class;
    use crate::character::enemy;
    use crate::randomizer::random;

    #[test]
    fn won() {
        let mut game = Game::new();
        // same level as player
        let enemy_class = class::Class::random(&class::Category::Common);
        let mut enemy = character::Character::new(enemy_class, 1);

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

        let enemy_class = class::Class::random(&class::Category::Common);
        let mut enemy = character::Character::new(enemy_class, 1);
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
        let enemy_class = class::Class::random(&class::Category::Common);
        let mut enemy = character::Character::new(enemy_class, 10);
        let result = game.battle(&mut enemy);
        assert!(result.is_err());
    }

    #[test]
    fn magic_attacks() {
        let mut game = Game::new();
        let mut enemy = enemy::at(&game.location, &game.player);
        enemy.max_hp = 100;
        enemy.current_hp = 100;

        game.player.change_class("mage").unwrap_or_default();
        game.player.max_mp = 10;
        game.player.current_mp = 10;
        game.player.strength = 10;

        // mage -mp with enough mp
        player_attack(&mut game, &mut enemy, &random());
        assert_eq!(7, game.player.current_mp);
        assert_eq!(70, enemy.current_hp);

        player_attack(&mut game, &mut enemy, &random());
        player_attack(&mut game, &mut enemy, &random());
        assert_eq!(1, game.player.current_mp);
        assert_eq!(10, enemy.current_hp);

        // mage -mp=0 without enough mp
        player_attack(&mut game, &mut enemy, &random());
        assert_eq!(1, game.player.current_mp);
        assert_eq!(7, enemy.current_hp);
    }
}
