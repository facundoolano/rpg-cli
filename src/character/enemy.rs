use super::{class::Category, class::Class, Character};
use crate::location;
use crate::randomizer::{random, Randomizer};
use rand::prelude::SliceRandom;

pub fn at(location: &location::Location, player: &Character) -> Character {
    let distance = location.distance_from_home();
    let level = level(player.level, distance.len());
    let category = weighted_choice(distance);
    Character::new(Class::random(&category), level)
}

fn level(player_level: i32, distance_from_home: i32) -> i32 {
    let level = std::cmp::max(player_level / 2 + distance_from_home - 1, 1);
    random().enemy_level(level)
}

/// Choose an enemy randomly, with higher chance to difficult enemies the further from home.
fn weighted_choice(distance: location::Distance) -> Category {
    // the weights for each group of enemies are different depending on the distance
    // the further from home, the bigger the chance to find difficult enemies
    let (w_common, w_rare, w_legendary) = match distance {
        location::Distance::Near(_) => (9, 2, 0),
        location::Distance::Mid(_) => (7, 10, 1),
        location::Distance::Far(_) => (1, 6, 3),
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

        // Player level 5
        assert_eq!(2, level(5, 1));
        assert_eq!(3, level(5, 2));
        assert_eq!(4, level(5, 3));

        // player level 10
        assert_eq!(5, level(10, 1));
        assert_eq!(6, level(10, 2));
        assert_eq!(7, level(10, 3));
    }
}
