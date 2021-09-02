use super::{class::Category, class::Class, Character};
use crate::item::ring::Ring;
use crate::location;
use crate::randomizer::{random, Randomizer};
use rand::prelude::SliceRandom;
use rand::Rng;

pub fn at(location: &location::Location, player: &Character) -> Character {
    // TODO refactor, put the class details in the function that defines if it appears or not
    let (class, level) = if should_find_gorthaur(player, location) {
        let mut class = Class::player_first().clone();
        class.name = String::from("gorthaur");
        class.hp.0 *= 2;
        class.strength.0 *= 2;
        class.category = Category::Legendary;
        (class, 100)
    } else if should_find_shadow(location) {
        let mut class = player.class.clone();
        class.name = String::from("shadow");
        class.category = Category::Rare;
        (class, player.level + 3)
    } else if should_find_dev(location) {
        let mut class = Class::player_first().clone();
        class.name = String::from("dev");
        class.hp.0 /= 2;
        class.strength.0 /= 2;
        class.speed.0 /= 2;
        class.category = Category::Rare;
        (class, player.level)
    } else {
        let distance = location.distance_from_home();
        let level = level(player.level, distance.len());
        let category = weighted_choice(distance);
        (Class::random(category).clone(), level)
    };

    Character::new(class, level)
}

fn level(player_level: i32, distance_from_home: i32) -> i32 {
    let level = std::cmp::max(player_level / 10 + distance_from_home - 1, 1);
    random().enemy_level(level)
}

fn should_find_gorthaur(player: &Character, location: &location::Location) -> bool {
    let wearing_ring =
        player.left_ring == Some(Ring::Ruling) || player.right_ring == Some(Ring::Ruling);
    wearing_ring && location.distance_from_home().len() >= 100
}

fn should_find_shadow(location: &location::Location) -> bool {
    let mut rng = rand::thread_rng();
    location.is_home() && rng.gen_ratio(1, 10)
}

fn should_find_dev(location: &location::Location) -> bool {
    let mut rng = rand::thread_rng();
    location.is_rpg_dir() && rng.gen_ratio(1, 10)
}

/// Choose an enemy randomly, with higher chance to difficult enemies the further from home.
fn weighted_choice(distance: location::Distance) -> Category {
    // the weights for each group of enemies are different depending on the distance
    // the further from home, the bigger the chance to find difficult enemies
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

    weights
        .as_slice()
        .choose_weighted(&mut rng, |(_c, weight)| *weight)
        .unwrap()
        .0
        .clone()
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
