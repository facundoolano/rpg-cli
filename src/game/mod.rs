extern crate dirs;

use crate::character;
use crate::character::Character;
use crate::event::Event;
use crate::item::Item;
use crate::location::Location;
use crate::quest::QuestList;
use crate::randomizer::random;
use crate::randomizer::Randomizer;
use chest::Chest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub mod battle;
pub mod chest;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Game {
    pub player: Character,
    pub location: Location,
    pub gold: i32,
    pub quests: QuestList,
    pub inventory: HashMap<String, Vec<Box<dyn Item>>>,
    pub tombstones: HashMap<String, Chest>,
    inspected: HashSet<Location>,
}

pub struct ItemNotFound;
pub enum ClassChangeError {
    NotFound,
    NotAtHome,
}

impl Game {
    pub fn new() -> Self {
        let quests = QuestList::new();
        Self {
            location: Location::home(),
            player: Character::player(),
            gold: 0,
            inventory: HashMap::new(),
            tombstones: HashMap::new(),
            inspected: HashSet::new(),
            quests,
        }
    }

    /// Remove the game data and reset this reference.
    /// Progress is preserved across games.
    pub fn reset(&mut self) {
        let mut new_game = Self::new();
        // preserve tombstones and quests across hero's lifes
        std::mem::swap(&mut new_game.tombstones, &mut self.tombstones);
        std::mem::swap(&mut new_game.quests, &mut self.quests);

        // remember last selected class
        new_game.player.change_class(&self.player.class.name).unwrap_or_default();

        // replace the current, finished game with the new one
        *self = new_game;
    }

    /// Move the hero's location towards the given destination, one directory
    /// at a time, with some chance of enemies appearing on each one.
    pub fn go_to(
        &mut self,
        dest: &Location,
        run: bool,
        bribe: bool,
    ) -> Result<(), character::Dead> {
        while self.location != *dest {
            self.visit(self.location.go_to(dest))?;

            if !self.location.is_home() {
                if let Some(mut enemy) = self.maybe_spawn_enemy() {
                    return self.maybe_battle(&mut enemy, run, bribe);
                }
            }
        }
        Ok(())
    }

    /// Look for chests and tombstones at the current location.
    /// Remembers previously visited locations for consistency.
    pub fn inspect(&mut self) {
        let maybe_tomb = self.tombstones.remove(&self.location.to_string());
        self.pick_up_chest(maybe_tomb, true);

        if !self.inspected.contains(&self.location) {
            self.inspected.insert(self.location.clone());
            self.pick_up_chest(Chest::generate(self), false);
        }
    }

    fn pick_up_chest(&mut self, maybe_chest: Option<Chest>, is_tombstone: bool) {
        if let Some(mut chest) = maybe_chest {
            let (items, gold) = chest.pick_up(self);
            Event::emit(
                self,
                Event::ChestFound {
                    items: &items,
                    gold,
                    is_tombstone,
                },
            );
        }
    }

    /// Set the hero's location to the one given, and apply related side effects.
    pub fn visit(&mut self, location: Location) -> Result<(), character::Dead> {
        self.location = location;
        if self.location.is_home() {
            let recovered = self.player.heal_full();
            let healed = self.player.maybe_remove_status_effect();
            Event::emit(
                self,
                Event::Heal {
                    item: None,
                    recovered,
                    healed,
                },
            );
        }

        // In location is home, already healed of negative status
        self.maybe_receive_status_damage()
    }

    /// Player takes damage from status_effects, if any.
    fn maybe_receive_status_damage(&mut self) -> Result<(), character::Dead> {
        if let Some(damage) = self.player.receive_status_effect_damage()? {
            Event::emit(self, Event::StatusEffectDamage { damage });
        }
        Ok(())
    }

    /// Set the current location to home, and apply related side-effects
    pub fn visit_home(&mut self) {
        self.visit(Location::home()).unwrap_or_default();
    }

    // TODO consider introducing an item "bag" wrapper over these types of hashmaps
    // (same is used in chests and in tests)
    pub fn add_item(&mut self, name: &str, item: Box<dyn Item>) {
        let entry = self
            .inventory
            .entry(name.to_string())
            .or_insert_with(Vec::new);
        entry.push(item);
    }

    pub fn use_item(&mut self, name: &str) -> Result<(), ItemNotFound> {
        let name = name.to_string();
        // get all items of that type and use one
        // if there are no remaining, drop the type from the inventory
        if let Some(mut items) = self.inventory.remove(&name) {
            if let Some(item) = items.pop() {
                item.apply(self);
                Event::emit(self, Event::ItemUsed { item: name.clone() });
            }

            if !items.is_empty() {
                self.inventory.insert(name, items);
            }

            Ok(())
        } else {
            Err(ItemNotFound)
        }
    }

    pub fn inventory(&self) -> HashMap<&str, usize> {
        self.inventory
            .iter()
            .map(|(k, v)| (k.as_ref(), v.len()))
            .collect::<HashMap<&str, usize>>()
    }

    pub fn change_class(&mut self, name: &str) -> Result<(), ClassChangeError> {
        if !self.location.is_home() {
            Err(ClassChangeError::NotAtHome)
        } else if let Ok(lost_xp) = self.player.change_class(name) {
            Event::emit(self, Event::ClassChanged { lost_xp });
            Ok(())
        } else {
            Err(ClassChangeError::NotFound)
        }
    }

    pub fn maybe_spawn_enemy(&mut self) -> Option<Character> {
        let distance = self.location.distance_from_home();
        if random().should_enemy_appear(&distance) {
            let level = enemy_level(self.player.level, distance.len());
            let level = random().enemy_level(level);
            let enemy = Character::enemy(level, distance);

            Event::emit(self, Event::EnemyAppears { enemy: &enemy });
            Some(enemy)
        } else {
            None
        }
    }

    pub fn maybe_battle(
        &mut self,
        enemy: &mut Character,
        run: bool,
        bribe: bool,
    ) -> Result<(), character::Dead> {
        // don't attempt bribe and run in the same turn
        if bribe {
            if self.bribe(enemy) {
                return Ok(());
            }
        } else if run && self.run_away(&enemy) {
            return Ok(());
        }

        self.battle(enemy)
    }

    fn bribe(&mut self, enemy: &Character) -> bool {
        let bribe_cost = gold_gained(self.player.level, enemy.level) / 2;

        if self.gold >= bribe_cost && random().bribe_succeeds() {
            self.gold -= bribe_cost;
            Event::emit(self, Event::Bribe { cost: bribe_cost });
            return true;
        };
        Event::emit(self, Event::Bribe { cost: 0 });
        false
    }

    fn run_away(&mut self, enemy: &Character) -> bool {
        let success = random().run_away_succeeds(
            self.player.level,
            enemy.level,
            self.player.speed,
            enemy.speed,
        );
        Event::emit(self, Event::RunAway { success });
        success
    }

    fn battle(&mut self, enemy: &mut Character) -> Result<(), character::Dead> {
        match battle::run(self, enemy, &random()) {
            Ok(xp) => {
                let gold = gold_gained(self.player.level, enemy.level);
                self.gold += gold;
                let levels_up = self.player.add_experience(xp);

                Event::emit(
                    self,
                    Event::BattleWon {
                        enemy: &enemy,
                        location: self.location.clone(),
                        xp,
                        levels_up,
                        gold,
                    },
                );

                if levels_up > 0 {
                    Event::emit(
                        self,
                        Event::LevelUp {
                            current: self.player.level,
                        },
                    )
                }

                Ok(())
            }
            Err(character::Dead) => {
                // Drop hero items in the location. If there was a previous tombstone
                // merge the contents of both chests
                let mut tombstone = Chest::drop(self);
                let location = self.location.to_string();
                if let Some(previous) = self.tombstones.remove(&location) {
                    tombstone.extend(previous);
                }
                self.tombstones.insert(location, tombstone);

                Event::emit(self, Event::BattleLost);
                Err(character::Dead)
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

fn enemy_level(player_level: i32, distance_from_home: i32) -> i32 {
    std::cmp::max(player_level / 2 + distance_from_home - 1, 1)
}

fn gold_gained(player_level: i32, enemy_level: i32) -> i32 {
    let level = std::cmp::max(1, enemy_level - player_level);
    random().gold_gained(level * 50)
}

#[cfg(test)]
mod tests {
    use crate::location::Distance;
    use item::equipment::Equipment;

    use super::*;
    use crate::item;
    use crate::randomizer;

    #[test]
    fn test_enemy_level() {
        // player level 1
        assert_eq!(1, enemy_level(1, 1));
        assert_eq!(1, enemy_level(1, 2));
        assert_eq!(2, enemy_level(1, 3));

        // Player level 5
        assert_eq!(2, enemy_level(5, 1));
        assert_eq!(3, enemy_level(5, 2));
        assert_eq!(4, enemy_level(5, 3));

        // player level 10
        assert_eq!(5, enemy_level(10, 1));
        assert_eq!(6, enemy_level(10, 2));
        assert_eq!(7, enemy_level(10, 3));
    }

    #[test]
    fn test_inventory() {
        let mut game = Game::new();

        assert_eq!(0, game.inventory().len());

        let potion = item::Potion::new(1);
        game.add_item("potion", Box::new(potion));
        assert_eq!(1, game.inventory().len());
        assert_eq!(1, *game.inventory().get("potion").unwrap());

        let potion = item::Potion::new(1);
        game.add_item("potion", Box::new(potion));
        assert_eq!(1, game.inventory().len());
        assert_eq!(2, *game.inventory().get("potion").unwrap());

        game.player.current_hp -= 3;
        assert_ne!(game.player.max_hp, game.player.current_hp);

        assert!(game.use_item("potion").is_ok());

        // check it actually restores the hp
        assert_eq!(game.player.max_hp, game.player.current_hp);

        // check item was consumed
        assert_eq!(1, game.inventory().len());
        assert_eq!(1, *game.inventory().get("potion").unwrap());

        assert!(game.use_item("potion").is_ok());
        assert_eq!(0, game.inventory().len());
        assert!(game.use_item("potion").is_err());
    }

    // NOTE: this tests are random and brittle and therefore bad unit tests but they
    // give a reasonable measure of how difficult the game is, so they are better than
    // nothing
    // TODO should do the same with fixed character classes
    // e.g. verify all classes can be beat at the game
    // FIXME ignoring as it's preventing builds
    #[ignore]
    #[test]
    fn test_not_unbeatable() {
        let times = 100;
        // The premise of this test is: a player with enough potions and its
        // level's equipment, should be able to beat any enemy of its same level
        // without relying in randomness.
        let (wins, lost_to) = run_battles_at(1, 1, times);
        assert_wins(times, wins, 0.5, &lost_to);

        let (wins, lost_to) = run_battles_at(1, 3, times);
        assert_wins(times, wins, 0.3, &lost_to);

        let (wins, lost_to) = run_battles_at(5, 5, times);
        assert_wins(times, wins, 0.7, &lost_to);

        let (wins, lost_to) = run_battles_at(10, 5, times);
        assert_wins(times, wins, 0.7, &lost_to);

        let (wins, lost_to) = run_battles_at(10, 10, times);
        assert_wins(times, wins, 0.4, &lost_to);

        let (wins, lost_to) = run_battles_at(15, 13, times);
        assert_wins(times, wins, 0.7, &lost_to);

        let (wins, lost_to) = run_battles_at(15, 15, times);
        assert_wins(times, wins, 0.5, &lost_to);

        let (wins, lost_to) = run_battles_at(50, 50, times);
        assert_wins(times, wins, 0.4, &lost_to);

        let (wins, lost_to) = run_battles_at(100, 100, times);
        assert_wins(times, wins, 0.4, &lost_to);

        // it shouldn't be too easy either --stronger enemies should have
        // chances of winning (even with all the equipment)
        let (wins, _) = run_battles_at(1, 6, times);
        assert_loses(times, wins, 0.2);

        let (wins, _) = run_battles_at(1, 10, times);
        assert_loses(times, wins, 0.3);

        let (wins, _) = run_battles_at(5, 10, times);
        assert_loses(times, wins, 0.2);

        let (wins, _) = run_battles_at(5, 15, times);
        assert_loses(times, wins, 0.35);

        let (wins, _) = run_battles_at(10, 15, times);
        assert_loses(times, wins, 0.15);

        let (wins, _) = run_battles_at(15, 20, times);
        assert_loses(times, wins, 0.15);

        let (wins, _) = run_battles_at(15, 20, times);
        assert_loses(times, wins, 0.15);

        let (wins, _) = run_battles_at(50, 60, times);
        assert_loses(times, wins, 0.15);
    }

    fn assert_wins(total: i32, wins: i32, expected_ratio: f64, lost_to: &Vec<String>) {
        assert!(
            wins as f64 >= total as f64 * expected_ratio,
            "won {} out of {}. Lost to {:?}",
            wins,
            total,
            lost_to
        );
    }

    fn assert_loses(total: i32, wins: i32, expected_ratio: f64) {
        let expected = (total as f64) * (1.0 - expected_ratio);
        assert!(
            (wins as f64) <= expected,
            "won {} out of {} expected at most {}",
            wins,
            total,
            expected
        );
    }

    fn run_battles_at(player_level: i32, distance: i32, times: i32) -> (i32, Vec<String>) {
        let mut wins = 0;
        let mut lost_to = Vec::new();

        // we don't want randomization turned off for this test
        let random = randomizer::DefaultRandomizer {};

        for _ in 0..times {
            let mut game = full_game_at(player_level);

            // duplicate randomization from the game
            let e_level = enemy_level(player_level, distance);
            let e_level = random.enemy_level(e_level);
            let mut enemy = Character::enemy(e_level, Distance::from(distance));

            if battle::run(&mut game, &mut enemy, &random).is_ok() {
                wins += 1
            } else {
                lost_to.push(format!("{}[{}]", enemy.name(), enemy.level));
            }
        }

        (wins, lost_to)
    }

    fn full_game_at(level: i32) -> Game {
        let mut game = Game::new();

        // get a player of the given level
        for _ in 0..level - 1 {
            game.player.add_experience(game.player.xp_for_next());
        }
        assert_eq!(level, game.player.level);

        // add potions of the given level
        for _ in 0..10 {
            game.add_item("potion", Box::new(item::Potion::new(level)));
        }

        // add equipment of the given level
        game.player.sword = Some(item::equipment::Sword::new(level));
        game.player.shield = Some(item::equipment::Shield::new(level));

        game
    }
}
