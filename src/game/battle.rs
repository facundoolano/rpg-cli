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
}
