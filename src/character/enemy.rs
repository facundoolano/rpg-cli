use super::{class::Class, Character};
use crate::location;
use crate::randomizer::{random, Randomizer};

pub fn at(location: &location::Location, player: &Character) -> Character {
    let level = level(player.level, location.distance_from_home().len());

    // TODO move random gen over here
    Character::new(Class::random_enemy(location.distance_from_home()), level)
}

pub fn level(player_level: i32, distance_from_home: i32) -> i32 {
    let base_level = std::cmp::max(player_level / 2 + distance_from_home - 1, 1);
    random().enemy_level(base_level)
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
