use super::{class::Category, class::Class, Character};
use crate::item::ring::Ring;
use crate::location;
use crate::log;
use crate::randomizer::{random, Randomizer};
use rand::prelude::SliceRandom;
use rand::Rng;

/// TODO
pub fn spawn(location: &location::Location, player: &Character) -> Option<Character> {
    if location.is_home() || player.enemies_evaded() {
        return None;
    }

    let distance = location.distance_from_home();

    if random().should_enemy_appear(&distance) {
        // try spawning "special" enemies if conditions are met, otherwise
        // a random one for the current location

        let (class, level) = spawn_gorthaur(player, location)
            .or_else(|| spawn_shadow(player, location))
            .or_else(|| spawn_dev(player, location))
            .unwrap_or_else(|| spawn_random(player, location));

        let enemy = Character::new(class, level);
        log::enemy_appears(&enemy, location);
        Some(enemy)
    } else {
        None
    }
}

fn spawn_gorthaur(player: &Character, location: &location::Location) -> Option<(Class, i32)> {
    let wearing_ring =
        player.left_ring == Some(Ring::Ruling) || player.right_ring == Some(Ring::Ruling);

    if wearing_ring && location.distance_from_home().len() >= 100 {
        let mut class = Class::player_first().clone();
        class.name = String::from("gorthaur");
        class.hp.0 *= 2;
        class.strength.0 *= 2;
        class.category = Category::Legendary;
        Some((class, 100))
    } else {
        None
    }
}

fn spawn_shadow(player: &Character, location: &location::Location) -> Option<(Class, i32)> {
    let mut rng = rand::thread_rng();
    if location.is_home() && rng.gen_ratio(1, 10) {
        let mut class = player.class.clone();
        class.name = String::from("shadow");
        class.category = Category::Rare;
        Some((class, player.level + 3))
    } else {
        None
    }
}

fn spawn_dev(player: &Character, location: &location::Location) -> Option<(Class, i32)> {
    let mut rng = rand::thread_rng();

    if location.is_rpg_dir() && rng.gen_ratio(1, 10) {
        let mut class = Class::player_first().clone();
        class.name = String::from("dev");
        class.hp.0 /= 2;
        class.strength.0 /= 2;
        class.speed.0 /= 2;
        class.category = Category::Rare;
        Some((class, player.level))
    } else {
        None
    }
}

/// Choose an enemy randomly, with higher chance to difficult enemies the further from home.
fn spawn_random(player: &Character, location: &location::Location) -> (Class, i32) {
    // the weights for each group of enemies are different depending on the distance
    // the further from home, the bigger the chance to find difficult enemies
    let distance = location.distance_from_home();
    let (w_common, w_rare, w_legendary) = match distance {
        location::Distance::Near(_) => (10, 2, 0),
        location::Distance::Mid(_) => (8, 10, 1),
        location::Distance::Far(_) => (0, 8, 2),
    };

    let mut rng = rand::thread_rng();

    // assign weights to each group and select one
    let weights = vec![
        (Category::Common, w_common),
        (Category::Rare, w_rare),
        (Category::Legendary, w_legendary),
    ];

    let category = weights
        .as_slice()
        .choose_weighted(&mut rng, |(_c, weight)| *weight)
        .unwrap()
        .0
        .clone();

    let level = std::cmp::max(player.level / 10 + distance.len() - 1, 1);
    let level = random().enemy_level(level);
    (Class::random(category).clone(), level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy_level() {
        // player level 1
        assert_eq!(1, level(1, 1));
        assert_eq!(1, level(1, 2));
        assert_eq!(2, level(1, 3));
        assert_eq!(9, level(1, 10));

        // Player level 5
        assert_eq!(1, level(5, 1));
        assert_eq!(1, level(5, 2));
        assert_eq!(2, level(5, 3));
        assert_eq!(9, level(5, 10));

        // player level 10
        assert_eq!(1, level(10, 1));
        assert_eq!(2, level(10, 2));
        assert_eq!(3, level(10, 3));
        assert_eq!(10, level(10, 10));
    }
}
