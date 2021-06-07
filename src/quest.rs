use core::fmt;
use crate::game;
use crate::character::Character;

#[typetag::serde(tag = "type")]
pub trait Quest: fmt::Display {
    /// What to show in the TODO quests list
    fn description (&self) -> String;

    /// Whether this quest should appear in the quest list
    fn is_visible(&self, _game: &game::Game) -> bool {
        true
    }

    /// Whether this quest should be listed as TODO or DONE
    fn is_done(&self) -> bool;

    /// The gold rewarded upon quest completion
    fn reward(&self) -> i32;

    // Event handlers.
    // By default do nothing since most will only need to override one.
    fn battle_won(&mut self, _enemy: &Character, _levels_up: i32) {}
    fn item_bought(&mut self, _name: &str) {}
    fn item_used(&mut self, _name: &str) {}
    fn tombstone(&mut self) {}
}

pub fn list() {
    todo!();
}

// EVENT HANDLING
// TODO this could be simplified by having each quest registering to individual events

// alias for quests passed dynamically
type QuestRef<'a> = &'a mut Box<dyn Quest>;

pub fn battle_won(game: &mut game::Game, enemy: &Character, levels_up: i32) {
    let handler = |q: QuestRef| q.battle_won(&enemy, levels_up);
    update(game, &handler);
}

pub fn item_bought(game: &mut game::Game, name: &str) {
    let handler = |q: QuestRef| q.item_bought(name);
    update(game, &handler);
}

pub fn item_used(game: &mut game::Game, name: &str) {
    let handler = |q: QuestRef| q.item_used(name);
    update(game, &handler);
}

pub fn tombstone(game: &mut game::Game) {
    let handler = |q: QuestRef| q.tombstone();
    update(game, &handler);
}

fn update(game: &mut game::Game, handler: &dyn Fn(QuestRef)) {
    for quest in game.quests.iter_mut() {
        handler(quest);
        if quest.is_done() {
            let reward = quest.reward();
            game.gold += reward;
            // TODO log::quest_done(reward);
        }
    }
}
