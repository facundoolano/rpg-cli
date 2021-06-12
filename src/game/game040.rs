use crate::character::Character;
use crate::item::Item;
use crate::location::Location;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::tombstone::Tombstone;

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
pub fn deserialize(data: &[u8]) -> Result<super::Game, bincode::Error> {
    let mut v4game: Game040 = bincode::deserialize(&data)?;
    let mut new_game = super::Game::new();
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
