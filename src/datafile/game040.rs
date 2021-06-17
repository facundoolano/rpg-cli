use crate::game;
use crate::game::tombstone::Tombstone;
use crate::character::Character;
use crate::item::Item;
use crate::location::Location;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// v0.4.0 of the game struct, kept for backwards compatibility when upgrading
/// the game data file to the new version
/// FIXME this will be removed on a subsequent version
#[derive(Serialize, Deserialize)]
struct Game040 {
    pub player: Character,
    pub location: Location,
    pub gold: i32,
    inventory: HashMap<String, Vec<Box<dyn Item>>>,
    tombstones: HashMap<Location, Tombstone>,
}

/// Get a new Game instance out of a v0.4.0 one
pub fn deserialize(data: &[u8]) -> Result<game::Game, bincode::Error> {
    let mut v4game: Game040 = bincode::deserialize(&data)?;
    let mut new_game = game::Game::new();
    std::mem::swap(&mut new_game.player, &mut v4game.player);
    std::mem::swap(&mut new_game.location, &mut v4game.location);
    std::mem::swap(&mut new_game.inventory, &mut v4game.inventory);
    new_game.tombstones = v4game
        .tombstones
        .drain()
        .map(|(l, t)| (l.to_string(), t))
        .collect();
    new_game.gold = v4game.gold;
    Ok(new_game)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item;
    use item::equipment::Equipment;

    #[test]
    fn test_backwards_compatibility() {
        // build a game040 with everything
        let mut game_v4 = Game040 {
            location: Location::home(),
            player: Character::player(),
            gold: 10,
            inventory: HashMap::new(),
            tombstones: HashMap::new(),
        };
        game_v4.player.sword = Some(item::equipment::Sword::new(1));
        game_v4.player.shield = Some(item::equipment::Shield::new(1));
        game_v4
            .inventory
            .insert("potion".to_string(), vec![Box::new(item::Potion::new(1))]);

        let mut tombstone_game = super::game::Game::new();
        tombstone_game.add_item("potion", Box::new(item::Potion::new(1)));
        game_v4
            .tombstones
            .insert(Location::home(), Tombstone::drop(&mut tombstone_game));

        let data = bincode::serialize(&game_v4).unwrap();
        let mut new_game = deserialize(&data).unwrap();

        assert_eq!(10, new_game.gold);
        assert!(new_game.location.is_home());
        assert!(new_game.player.sword.is_some());
        assert!(new_game.player.shield.is_some());
        assert_eq!(1 as usize, *new_game.inventory().get("potion").unwrap());
        // pick up tombstone @ home
        new_game.inspect();
        assert_eq!(2 as usize, *new_game.inventory().get("potion").unwrap());
    }
}
