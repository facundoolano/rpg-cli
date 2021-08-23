use std::{collections::HashMap, fmt::Display};

use super::equipment::Equipment;
use crate::character::Character;
use crate::event::Event;
use crate::game::Game;
use super::key::Key;
use crate::log;
use super::Item;
use anyhow::{bail, Result};

/// Print the list of available items and their price.
pub fn list(game: &Game) -> Result<()> {
    if !game.location.is_home() {
        bail!("Shop is only allowed at home.");
    }

    let items = available_items(&game.player).iter().map(|s| (s.cost(), s.to_string())).collect();
    log::shop_list(game, items);
    Ok(())
}

/// Buy an item and add it to the game.
pub fn buy(game: &mut Game, item: Key) -> Result<()> {
    if !game.location.is_home() {
        bail!("Shop is only allowed at home.");
    }

    let player = &mut game.player;
    let mut items = available_items(player)
        .into_iter()
        .map(|s| (s.to_key(), s))
        .collect::<HashMap<Key, Box<dyn Shoppable>>>();

    if let Some(item) = items.remove(&item) {
        item.buy(game)?;
        Ok(())
    } else {
        bail!("Item not available.")
    }
}

// FIXME remove superfluous maps
/// Build a list of items currently available at the shop
fn available_items(player: &Character) -> Vec<Box<dyn Shoppable>> {
    let mut items = Vec::<Box<dyn Shoppable>>::new();
    let level = player.rounded_level();

    let sword = Equipment::Sword(level);
    if sword.is_upgrade_from(&player.sword) {
        items.push(Box::new(sword));
    }

    let shield = Equipment::Shield(level);
    if shield.is_upgrade_from(&player.shield) {
        items.push(Box::new(shield));
    }

    let potion = super::Potion::new(level);
    items.push(Box::new(potion));

    let ether = super::Ether::new(level);
    items.push( Box::new(ether));

    let remedy = super::Remedy::new();
    items.push(Box::new(remedy));

    let escape = super::Escape::new();
    items.push(Box::new(escape));

    items
}

// FIXME remove trait or implement generically for item
trait Shoppable: Display {
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
    fn to_key(&self) -> Key;
}

impl Shoppable for Equipment {
    fn cost(&self) -> i32 {
        self.level() * 500
    }

    fn add_to(&self, game: &mut Game) {
        match self {
            Equipment::Sword(_) => game.player.sword = Some(self.clone()),
            Equipment::Shield(_) => game.player.shield = Some(self.clone()),
        }
    }

    fn to_key(&self) -> Key {
        match self {
            Equipment::Sword(_) => Key::Sword,
            Equipment::Shield(_) => Key::Shield,
        }
    }
}

// FIXME see if the add_to/key repetition can be removed
impl Shoppable for super::Potion {
    fn cost(&self) -> i32 {
        self.level * 200
    }

    fn add_to(&self, game: &mut Game) {
        game.add_item(Box::new(self.clone()));
    }

    fn to_key(&self) -> Key {
        self.key()
    }
}

impl Shoppable for super::Escape {
    fn cost(&self) -> i32 {
        1000
    }

    fn add_to(&self, game: &mut Game) {
        game.add_item(Box::new(self.clone()));
    }

    fn to_key(&self) -> Key {
        self.key()
    }
}

impl Shoppable for super::Remedy {
    fn cost(&self) -> i32 {
        400
    }

    fn add_to(&self, game: &mut Game) {
        game.add_item(Box::new(self.clone()));
    }

    fn to_key(&self) -> Key {
        self.key()
    }
}

impl Shoppable for super::Ether {
    fn cost(&self) -> i32 {
        self.level * 250
    }

    fn add_to(&self, game: &mut Game) {
        game.add_item(Box::new(self.clone()));
    }

    fn to_key(&self) -> Key {
        self.key()
    }
}
