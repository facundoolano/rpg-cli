use std::{collections::HashMap, fmt::Display};

use super::equipment::{Equipment, Shield, Sword};
use crate::character::Character;
use crate::game::Game;
use crate::log;

pub enum Error {
    NotEnoughGold,
    ItemNotAvailable,
}

/// Print the list of available items and their price.
pub fn list(game: &Game) {
    let items = available_items(&game.player)
        .into_iter()
        .map(|(_, item)| item)
        .collect::<Vec<Box<dyn Shoppable>>>();
    log::shop_list(game, items);
}

/// Buy an item and add it to the game.
pub fn buy(game: &mut Game, item: &str) -> Result<(), Error> {
    let player = &mut game.player;
    let mut items = available_items(player)
        .into_iter()
        .collect::<HashMap<String, Box<dyn Shoppable>>>();
    if let Some(item) = items.remove(item) {
        item.buy(game)?;
        Ok(())
    } else {
        Err(Error::ItemNotAvailable)
    }
}

/// Build a list of items currently available at the shop
fn available_items(player: &Character) -> Vec<(String, Box<dyn Shoppable>)> {
    let mut items = Vec::<(String, Box<dyn Shoppable>)>::new();
    let level = available_level(&player);

    if can_upgrade_sword(player, level) {
        items.push(("sword".to_string(), Box::new(Sword::new(level))));
    }

    if can_upgrade_shield(player, level) {
        items.push(("shield".to_string(), Box::new(Shield::new(level))));
    }

    let potion = super::Potion::new(level);
    items.push(("potion".to_string(), Box::new(potion)));

    let escape = super::Escape::new();
    items.push(("escape".to_string(), Box::new(escape)));

    items
}

/// The offered items/equipment have levels e.g. potion[1], sword[5], etc.
/// they become available for purchase only when the player reaches that level
fn available_level(player: &Character) -> i32 {
    // allow level 1 or level 5n
    std::cmp::max(1, (player.level / 5) * 5)
}

fn can_upgrade_sword(player: &Character, level: i32) -> bool {
    player.sword.is_none() || player.sword.as_ref().unwrap().level() < level
}

// duplicated but can't find a reasonable alternative
fn can_upgrade_shield(player: &Character, level: i32) -> bool {
    player.shield.is_none() || player.shield.as_ref().unwrap().level() < level
}

pub trait Shoppable: Display {
    fn cost(&self) -> i32;
    fn buy(&self, game: &mut Game) -> Result<(), Error> {
        if game.gold < self.cost() {
            return Err(Error::NotEnoughGold);
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
