use std::collections::HashMap;
use std::fmt::Display;

use super::{Equipment, Shield, Sword};
use crate::character::Character;
use crate::game::Game;

/// Print the list of available items and their price.
pub fn list(player: &Character) {
    for item in available_items(player).values() {
        println!("{}  {}g", item, item.cost());
    }
}

/// Buy an item and add it to the game.
pub fn buy(game: &mut Game, item: &str) -> Result<(), String> {
    let player = &mut game.player;
    let mut items = available_items(player);
    if let Some(item) = items.remove(&item.to_lowercase()) {
        item.buy(game)?;
        Ok(())
    } else {
        Err("item not available".to_string())
    }
}

/// Build a list of items currently available at the shop
fn available_items(player: &Character) -> HashMap<String, Box<dyn Shoppable>> {
    let mut items = HashMap::<String, Box<dyn Shoppable>>::new();
    let level = available_level(&player);

    if player.sword.is_none() || player.sword.as_ref().unwrap().level < level {
        items.insert("sword".to_string(), Box::new(Sword::new(level)));
    }

    if player.shield.is_none() || player.shield.as_ref().unwrap().level < level {
        items.insert("shield".to_string(), Box::new(Shield::new(level)));
    }

    let potion = super::Potion::new(level);
    items.insert("potion".to_string(), Box::new(potion));

    let escape = super::Escape::new();
    items.insert("escape".to_string(), Box::new(escape));

    items
}

/// The offered items/equipment have levels e.g. potion[1], sword[5], etc.
/// they become available for purchase only when the player reaches that level
fn available_level(player: &Character) -> i32 {
    // allow level 1 or level 5n
    std::cmp::max(1, (player.level / 5) * 5)
}

trait Shoppable: Display {
    fn cost(&self) -> i32;
    fn buy(&self, game: &mut Game) -> Result<(), String> {
        if game.gold < self.cost() {
            return Err("Not enough gold".to_string());
        }
        game.gold -= self.cost();
        self.add_to(game);
        Ok(())
    }
    fn add_to(&self, game: &mut Game);
}

impl Shoppable for Sword {
    fn cost(&self) -> i32 {
        self.level() * 500
    }

    fn add_to(&self, game: &mut Game) {
        game.player.sword = Some(self.clone())
    }
}

impl Shoppable for Shield {
    fn cost(&self) -> i32 {
        self.level() * 500
    }

    fn add_to(&self, game: &mut Game) {
        game.player.shield = Some(self.clone())
    }
}

impl Shoppable for super::Potion {
    fn cost(&self) -> i32 {
        self.level * 200
    }

    fn add_to(&self, game: &mut Game) {
        game.add_item("potion", Box::new(self.clone()));
    }
}

impl Shoppable for super::Escape {
    fn cost(&self) -> i32 {
        1000
    }

    fn add_to(&self, game: &mut Game) {
        game.add_item("escape", Box::new(self.clone()));
    }
}
