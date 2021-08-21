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

    let items: Vec<_> = available_items(&game.player)
        .into_iter()
        .map(|(_, item)| item)
        .collect();
    log::shop_list(game, items);
    Ok(())
}

/// Atomically buy all the items and add them to the game or none of them
/// if any item can't be bought (not enough gold or not available).
pub fn buy(game: &mut Game, items: &[String]) -> Result<()> {
    if !game.location.is_home() {
        bail!("Shop is only allowed at home.");
    }

    let player = &mut game.player;
    let mut shop_items: HashMap<_, _> = available_items(player).into_iter().collect();

    // We check that we have enough golds and that all of the items we try to buy are available.
    let mut total_cost = 0;
    let mut unavailable_items = Vec::new();
    let mut tmp_shop_items: HashMap<_, _> = available_items(player).into_iter().collect();
    for item in items {
        match tmp_shop_items.remove(item.as_str()) {
            Some(item) => total_cost += item.cost(),
            None => unavailable_items.push(item.as_str()),
        }
    }

    if !unavailable_items.is_empty() {
        let count = unavailable_items.len();
        let verb = if count == 1 { "item is" } else { "are" };
        let items = unavailable_items.join(", ");
        bail!("{} {} unavailable.", items, verb);
    }

    if total_cost > game.gold {
        bail!("Not enough gold.");
    }

    for item in items {
        if let Some(item) = shop_items.remove(item.as_str()) {
            item.buy(game)?;
        }
    }

    Ok(())
}

/// Build a list of items currently available at the shop
fn available_items(player: &Character) -> Vec<(&'static str, Box<dyn Shoppable>)> {
    let mut items = Vec::<(_, Box<dyn Shoppable>)>::new();
    let level = player.rounded_level();

    let sword = Weapon::Sword(level);
    if sword.is_upgrade_from(&player.equip.sword) {
        items.push(("sword", Box::new(sword)));
    }

    let shield = Weapon::Shield(level);
    if shield.is_upgrade_from(&player.equip.shield) {
        items.push(("shield", Box::new(shield)));
    }

    let potion = super::Potion::new(level);
    items.push(("potion", Box::new(potion)));

    let ether = super::Ether::new(level);
    items.push(("ether", Box::new(ether)));

    let remedy = super::Remedy::new();
    items.push(("remedy", Box::new(remedy)));

    let escape = super::Escape::new();
    items.push(("escape", Box::new(escape)));

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
            Weapon::Sword(_) => game.player.equip.sword = Some(self.clone()),
            Weapon::Shield(_) => game.player.equip.shield = Some(self.clone()),
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
