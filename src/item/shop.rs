use std::fmt::Display;

use super::equipment::Equipment;
use super::key::Key;
use super::Item;
use crate::character::Character;
use crate::game::Game;
use crate::log;
use crate::quest;
use anyhow::{bail, Result};
use std::collections::HashMap;

/// Print the list of available items and their price.
pub fn list(game: &Game) -> Result<()> {
    if !game.location.is_home() {
        bail!("Shop is only allowed at home.");
    }

    let items = available_items(&game.player)
        .iter()
        .map(|s| (s.cost(), s.to_string()))
        .collect();
    log::shop_list(game, items);
    Ok(())
}

/// Buy as much as possible from the given item list.
/// Will stop buying if there's an error (ran out of money or requested item is
/// not available), but will keep the shopped items so far.
/// Will bail on error only after reporting what was bought.
pub fn buy(game: &mut Game, item_keys: &[Key]) -> Result<()> {
    if !game.location.is_home() {
        bail!("Shop is only allowed at home.");
    }

    let mut item_counts = HashMap::new();
    let mut total_cost = 0;
    let mut error = String::from("");

    // Buy one at a time and break on first error
    for key in item_keys {
        // get list every time to prevent e.g. buying the sword twice
        let item = available_items(&game.player)
            .into_iter()
            .find(|s| s.to_key() == *key);

        if let Some(item) = item {
            let item_cost = item.cost();

            if game.gold < item_cost {
                error = "Not enough gold.".to_string();
                break;
            }
            game.gold -= item_cost;
            item.add_to(game);

            total_cost += item_cost;
            *item_counts.entry(key.clone()).or_insert(0) += 1;
            quest::item_bought(game, item.to_string());
        } else {
            error = format!("{} not available.", key);
            break;
        }
    }

    // log what could be bought even if there was an error
    log::shop_buy(total_cost, &item_counts);
    if !error.is_empty() {
        bail!(error);
    }
    Ok(())
}

/// Build a list of items currently available at the shop
fn available_items(player: &Character) -> Vec<Box<dyn Shoppable>> {
    let mut items = Vec::<Box<dyn Shoppable>>::new();
    let level = player.rounded_level();

    let sword = Equipment::sword(level);
    if sword.is_upgrade_from(&player.sword) {
        items.push(Box::new(sword));
    }

    let shield = Equipment::shield(level);
    if shield.is_upgrade_from(&player.shield) {
        items.push(Box::new(shield));
    }

    let potion = super::Potion::new(level);
    items.push(Box::new(potion));

    let ether = super::Ether::new(level);
    items.push(Box::new(ether));

    let remedy = super::Remedy::new();
    items.push(Box::new(remedy));

    let escape = super::Escape::new();
    items.push(Box::new(escape));

    items
}

trait Shoppable: Display {
    fn cost(&self) -> i32;
    fn add_to(&self, game: &mut Game);
    fn to_key(&self) -> Key;
}

impl Shoppable for Equipment {
    fn cost(&self) -> i32 {
        self.level() * 500
    }

    fn add_to(&self, game: &mut Game) {
        match self.key() {
            Key::Sword => game.player.sword = Some(self.clone()),
            Key::Shield => game.player.shield = Some(self.clone()),
            _ => {}
        }
    }

    fn to_key(&self) -> Key {
        self.key()
    }
}

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

#[cfg(test)]
mod tests {
    use super::super::Potion;
    use super::*;

    #[test]
    fn buy_one() {
        let potion = Potion::new(1);
        assert_eq!(200, potion.cost());

        let mut game = Game::new();
        game.gold = 1000;

        let result = buy(&mut game, &vec![Key::Potion]);
        assert!(result.is_ok());
        assert_eq!(800, game.gold);
        assert_eq!(1, *game.inventory().get(&Key::Potion).unwrap());
    }

    #[test]
    fn buy_multiple() {
        let mut game = Game::new();
        game.gold = 1000;

        let result = buy(&mut game, &vec![Key::Potion, Key::Potion, Key::Potion]);
        assert!(result.is_ok());
        assert_eq!(400, game.gold);
        assert_eq!(3, *game.inventory().get(&Key::Potion).unwrap());
    }

    #[test]
    fn buy_until_no_money() {
        let mut game = Game::new();
        game.gold = 500;

        let result = buy(&mut game, &vec![Key::Potion, Key::Potion, Key::Potion]);
        assert!(result.is_err());
        assert_eq!(100, game.gold);
        assert_eq!(2, *game.inventory().get(&Key::Potion).unwrap());
    }

    #[test]
    fn buy_until_not_available() {
        let mut game = Game::new();
        game.gold = 1000;

        // not sellable
        let result = buy(&mut game, &vec![Key::Potion, Key::MagicStone, Key::Potion]);
        assert!(result.is_err());
        assert_eq!(800, game.gold);
        assert_eq!(1, *game.inventory().get(&Key::Potion).unwrap());

        // sellable once, then unavailable
        let mut game = Game::new();
        game.gold = 2000;
        let result = buy(
            &mut game,
            &vec![Key::Potion, Key::Shield, Key::Shield, Key::Potion],
        );
        assert!(result.is_err());
        // 200 from potion - 500 from shield (once)
        assert_eq!(1300, game.gold);
        assert_eq!(1, *game.inventory().get(&Key::Potion).unwrap());
        assert!(game.player.shield.is_some());
    }
}
