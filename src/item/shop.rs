use std::{collections::HashMap, fmt::Display};

use super::equipment::Weapon;
use crate::character::Character;
use crate::event::Event;
use crate::game::Game;
use crate::log;
use anyhow::{bail, Result};

/// Print the list of available items and their price.
pub fn list(game: &Game) -> Result<()> {
    if !game.location.is_home() {
        bail!("Shop is only allowed at home.");
    }

    let items = available_items(&game.player)
        .into_iter()
        .map(|(_, item)| item)
        .collect::<Vec<Box<dyn Shoppable>>>();
    log::shop_list(game, items);
    Ok(())
}

/// Buy an item and add it to the game.
pub fn buy(game: &mut Game, item: &str) -> Result<()> {
    if !game.location.is_home() {
        bail!("Shop is only allowed at home.");
    }

    let player = &mut game.player;
    let mut items = available_items(player)
        .into_iter()
        .collect::<HashMap<String, Box<dyn Shoppable>>>();
    if let Some(item) = items.remove(item) {
        item.buy(game)?;
        Ok(())
    } else {
        bail!("Item not available.")
    }
}

/// Build a list of items currently available at the shop
fn available_items(player: &Character) -> Vec<(String, Box<dyn Shoppable>)> {
    let mut items = Vec::<(String, Box<dyn Shoppable>)>::new();
    let level = player.rounded_level();

    let sword = Weapon::Sword(level);
    if sword.is_upgrade_from(&player.sword) {
        items.push(("sword".to_string(), Box::new(sword)));
    }

    let shield = Weapon::Shield(level);
    if shield.is_upgrade_from(&player.shield) {
        items.push(("shield".to_string(), Box::new(shield)));
    }

    let potion = super::Potion::new(level);
    items.push(("potion".to_string(), Box::new(potion)));

    let ether = super::Ether::new(level);
    items.push(("ether".to_string(), Box::new(ether)));

    let remedy = super::Remedy::new();
    items.push(("remedy".to_string(), Box::new(remedy)));

    let escape = super::Escape::new();
    items.push(("escape".to_string(), Box::new(escape)));

    items
}

pub trait Shoppable: Display {
    fn cost(&self) -> i32;
    fn buy(&self, game: &mut Game) -> Result<()> {
        if game.gold < self.cost() {
            bail!("Not enough gold.");
        }
        game.gold -= self.cost();
        self.add_to(game);

        Event::emit(
            game,
            Event::ItemBought {
                item: self.to_string(),
            },
        );

        Ok(())
    }
    fn add_to(&self, game: &mut Game);
}

impl Shoppable for Weapon {
    fn cost(&self) -> i32 {
        self.level() * 500
    }

    fn add_to(&self, game: &mut Game) {
        match self {
            Weapon::Sword(_) => game.player.sword = Some(self.clone()),
            Weapon::Shield(_) => game.player.shield = Some(self.clone()),
        }
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

impl Shoppable for super::Remedy {
    fn cost(&self) -> i32 {
        400
    }

    fn add_to(&self, game: &mut Game) {
        game.add_item("remedy", Box::new(self.clone()));
    }
}

impl Shoppable for super::Ether {
    fn cost(&self) -> i32 {
        self.level * 250
    }

    fn add_to(&self, game: &mut Game) {
        game.add_item("ether", Box::new(self.clone()));
    }
}
