use crate::character::{Character, StatusEffect};
use crate::event::Event;
use crate::game::battle::AttackType;
use crate::game::Game;
use crate::item::shop;
use crate::location::Location;
use colored::*;
use once_cell::sync::OnceCell;

// This are initialized based on input args and then act as constants
// this prevents having to pass around the flags or lazily parsing the opts
static QUIET: OnceCell<bool> = OnceCell::new();
static PLAIN: OnceCell<bool> = OnceCell::new();

/// Set the global output preferences
pub fn init(quiet: bool, plain: bool) {
    QUIET.set(quiet).unwrap();
    PLAIN.set(plain).unwrap();
}

fn quiet() -> bool {
    *QUIET.get().unwrap_or(&false)
}

fn plain() -> bool {
    *PLAIN.get().unwrap_or(&false)
}

pub fn handle(game: &Game, event: &Event) {
    match event {
        Event::EnemyAppears { enemy } => {
            enemy_appears(enemy, &game.location);
        }
        Event::PlayerAttack {
            enemy,
            kind,
            damage,
            mp_cost,
        } => {
            attack(enemy, kind, *damage, *mp_cost);
        }
        Event::EnemyAttack {
            kind,
            damage,
            mp_cost,
        } => {
            attack(&game.player, kind, *damage, *mp_cost);
        }
        Event::StatusEffectDamage { damage } => {
            status_effect_damage(&game.player, *damage);
        }
        Event::BattleWon {
            xp,
            levels_up,
            gold,
            items,
            ..
        } => {
            battle_won(&game, *xp, *levels_up, *gold, items);
        }
        Event::BattleLost => {
            battle_lost(&game.player);
        }
        Event::ChestFound {
            is_tombstone: false,
            items,
            gold,
        } => {
            chest(items, *gold);
        }
        Event::ChestFound {
            is_tombstone: true,
            items,
            gold,
        } => {
            tombstone(items, *gold);
        }
        Event::Bribe { cost } => {
            bribe(&game.player, *cost);
        }
        Event::RunAway { success } => {
            run_away(&game.player, *success);
        }
        Event::Heal {
            item: Some(item),
            recovered_hp,
            recovered_mp,
            healed,
        } => {
            heal_item(&game.player, item, *recovered_hp, *recovered_mp, *healed);
        }
        Event::Heal {
            item: None,
            recovered_hp,
            recovered_mp,
            healed,
        } => {
            heal(
                &game.player,
                &game.location,
                *recovered_hp,
                *recovered_mp,
                *healed,
            );
        }
        Event::ClassChanged { lost_xp } => change_class(&game.player, &game.location, *lost_xp),
        Event::LevelUp { .. } => {}
        Event::ItemBought { .. } => {}
        Event::ItemUsed { .. } => {}
        Event::GameReset => {}
    }
}

/// Print the hero status according to options
pub fn status(game: &Game) {
    if plain() {
        plain_status(game);
    } else if quiet() {
        short_status(game);
    } else {
        long_status(game)
    }
}

pub fn shop_list(game: &Game, items: Vec<Box<dyn shop::Shoppable>>) {
    for item in items {
        let display = format!("{}", item);
        println!("    {:<10}  {}", display, format_gold(item.cost()));
    }

    println!("\n    funds: {}", format_gold(game.gold));
}

pub fn quest_list(todo: &[String], done: &[String]) {
    for quest in todo {
        println!("  {} {}", "□".dimmed(), quest);
    }
    for quest in done {
        println!("  {} {}", "✔".green(), quest.dimmed());
    }
}

pub fn quest_done(reward: i32) {
    if !quiet() {
        println!("    {} quest completed!", format_gold_plus(reward));
    }
}

fn enemy_appears(enemy: &Character, location: &Location) {
    log(enemy, location, "");
}

fn bribe(player: &Character, amount: i32) {
    if amount > 0 {
        let suffix = format!("bribed {}", format!("-{}g", amount).yellow());
        battle_log(player, &suffix);
    } else {
        battle_log(player, "can't bribe!");
    }
}

fn run_away(player: &Character, success: bool) {
    if success {
        battle_log(player, "fled!");
    } else {
        battle_log(player, "can't run!");
    }
}

fn heal(
    player: &Character,
    location: &Location,
    recovered_hp: i32,
    recovered_mp: i32,
    healed: bool,
) {
    let mut recovered_text = String::new();
    let mut healed_text = String::new();
    let mut mp_text = String::new();

    if recovered_hp > 0 {
        recovered_text = format!("+{}hp ", recovered_hp);
    }
    if recovered_mp > 0 {
        mp_text = format!("+{}mp ", recovered_mp);
    }
    if healed {
        healed_text = String::from("+healed ");
    }
    if recovered_hp > 0 || recovered_mp > 0 || healed {
        log(
            player,
            location,
            &format!(
                "{}{}{}",
                recovered_text.green(),
                mp_text.purple(),
                healed_text.purple()
            ),
        );
    }
}

fn heal_item(player: &Character, item: &str, recovered_hp: i32, recovered_mp: i32, healed: bool) {
    if recovered_hp > 0 {
        battle_log(
            player,
            &format!("+{}hp {}", recovered_hp, item).green().to_string(),
        );
    }
    if recovered_mp > 0 {
        battle_log(
            player,
            &format!("+{}mp {}", recovered_mp, item).purple().to_string(),
        );
    }
    if healed {
        battle_log(player, &format!("+healed {}", item).green());
    }
}

fn change_class(player: &Character, location: &Location, lost_xp: i32) {
    let mut lost_text = String::new();
    if lost_xp > 0 {
        lost_text = format!("-{}xp", lost_xp).bright_red().to_string();
    }
    log(player, location, &lost_text);
}

fn attack(character: &Character, attack: &AttackType, damage: i32, mp_cost: i32) {
    if !quiet() {
        battle_log(
            character,
            &format_attack(character, &attack, damage, mp_cost),
        );
    }
}

fn status_effect_damage(character: &Character, damage: i32) {
    let (_, emoji) = status_effect_params(character.status_effect.unwrap());
    battle_log(character, &format_damage(character, damage, &emoji));
}

fn battle_lost(player: &Character) {
    battle_log(player, "\u{1F480}");
}

fn battle_won(game: &Game, xp: i32, levels_up: i32, gold: i32, items: &[String]) {
    let level_str = if levels_up > 0 {
        let plus = (0..levels_up).map(|_| "+").collect::<String>();
        format!(" {}level", plus).cyan().to_string()
    } else {
        "".to_string()
    };

    battle_log(
        &game.player,
        &format!(
            "{}{}{}",
            format!("+{}xp", xp).bold(),
            level_str,
            format_ls("", items, gold)
        ),
    );
    short_status(game);
}

fn long_status(game: &Game) {
    let player = &game.player;
    let location = &game.location;

    println!("{}@{}", format_character(player), location);
    println!(
        "    hp:{} {}/{}",
        hp_display(player, 10),
        player.current_hp,
        player.max_hp
    );

    let (current_mp, max_mp) = if player.class.is_magic() {
        (player.current_mp, player.max_mp)
    } else {
        (0, 0)
    };
    println!(
        "    mp:{} {}/{}",
        mp_display(player, 10),
        current_mp,
        max_mp
    );

    println!(
        "    xp:{} {}/{}",
        xp_display(player, 10),
        player.xp,
        player.xp_for_next()
    );
    if let Some(status) = player.status_effect {
        println!("    status: {}", format_status_effect(status).bright_red());
    }
    println!(
        "    att:{}   mag:{}   def:{}   spd:{}",
        player.physical_attack(),
        player.magic_attack(),
        player.deffense(),
        player.speed
    );
    println!("    {}", format_equipment(player));
    println!("    {}", format_inventory(game));
    println!("    {}", format_gold(game.gold));
}

fn short_status(game: &Game) {
    let player = &game.player;

    let suffix = if let Some(status) = player.status_effect {
        let (_, emoji) = status_effect_params(status);
        emoji
    } else {
        ""
    };
    log(player, &game.location, &suffix);
}

fn plain_status(game: &Game) {
    let player = &game.player;

    let status_effect = if let Some(status) = player.status_effect {
        let (name, _) = status_effect_params(status);
        format!("status:{}\t", name)
    } else {
        String::new()
    };

    println!(
        "{}[{}]\t@{}\thp:{}/{}\tmp:{}/{}\txp:{}/{}\tatt:{}\tmag:{}\tdef:{}\tspd:{}\t{}{}\t{}\tg:{}",
        player.name(),
        player.level,
        game.location,
        player.current_hp,
        player.max_hp,
        player.current_mp,
        player.max_mp,
        player.xp,
        player.xp_for_next(),
        player.magic_attack(),
        player.physical_attack(),
        player.deffense(),
        player.speed,
        status_effect,
        format_equipment(player),
        format_inventory(game),
        game.gold
    );
}

fn chest(items: &[String], gold: i32) {
    println!("{}", format_ls("\u{1F4E6}", items, gold));
}

fn tombstone(items: &[String], gold: i32) {
    println!("{}", format_ls("\u{1FAA6} ", items, gold));
}

fn format_ls(emoji: &str, items: &[String], gold: i32) -> String {
    let mut string = format!("{} ", emoji);

    if gold > 0 {
        string.push_str(&format!("{} ", format_gold_plus(gold)));
    }
    for item in items {
        string.push_str(&format!("+{} ", item));
    }
    string
}

// HELPERS

/// Generic log function. At the moment all output of the game is structured as
/// of a player status at some location, with an optional event suffix.
fn log(character: &Character, location: &Location, suffix: &str) {
    println!(
        "{}{}{}{}@{} {}",
        format_character(character),
        hp_display(character, 4),
        mp_display(character, 4),
        xp_display(character, 4),
        location,
        suffix
    );
}

fn battle_log(character: &Character, suffix: &str) {
    println!(
        "{}{} {}",
        format_character(character),
        hp_display(character, 4),
        suffix
    );
}

fn format_character(character: &Character) -> String {
    let name = format!("{:>8}", character.name());
    let name = if character.is_player() {
        name.bold()
    } else {
        name.yellow().bold()
    };
    format!("{}[{}]", name, character.level)
}

fn format_equipment(character: &Character) -> String {
    let mut fragments = Vec::new();

    if let Some(sword) = &character.sword {
        fragments.push(sword.to_string());
    }

    if let Some(shield) = &character.shield {
        fragments.push(shield.to_string());
    }
    format!("equip:{{{}}}", fragments.join(","))
}

pub fn format_inventory(game: &Game) -> String {
    let mut items = game
        .inventory()
        .iter()
        .map(|(k, v)| format!("{}x{}", k, v))
        .collect::<Vec<String>>();

    items.sort();
    format!("item:{{{}}}", items.join(","))
}

fn format_attack(receiver: &Character, attack: &AttackType, damage: i32, mp_cost: i32) -> String {
    let magic_effect = if mp_cost > 0 { "\u{2728}" } else { "" };

    match attack {
        AttackType::Regular => format_damage(receiver, damage, &magic_effect),
        AttackType::Critical => {
            format_damage(receiver, damage, &format!("{} critical!", magic_effect))
        }
        AttackType::Effect(status_effect) => {
            format_damage(receiver, damage, &format_status_effect(*status_effect))
        }
        AttackType::Miss => format!("{} dodged!", magic_effect),
    }
}

fn format_damage(receiver: &Character, amount: i32, suffix: &str) -> String {
    let color = if receiver.is_player() {
        "bright red".to_string()
    } else {
        "white".to_string()
    };
    format!("-{}hp {}", amount, suffix).color(color).to_string()
}

fn format_status_effect(status_effect: StatusEffect) -> String {
    let (name, emoji) = status_effect_params(status_effect);
    format!("{} {}!", emoji, name)
}

fn status_effect_params(status_effect: StatusEffect) -> (&'static str, &'static str) {
    match status_effect {
        StatusEffect::Burning => ("burning", "\u{1F525}"),
        StatusEffect::Poisoned => ("poisoned", "\u{2620}\u{FE0F} "),
    }
}

fn hp_display(character: &Character, slots: i32) -> String {
    bar_display(
        slots,
        character.current_hp,
        character.max_hp,
        "green",
        "red",
    )
}

fn mp_display(character: &Character, slots: i32) -> String {
    let current_mp = if character.class.is_magic() {
        character.current_mp
    } else {
        0
    };

    bar_display(
        slots,
        current_mp,
        character.max_mp,
        "purple",
        "bright black",
    )
}

fn xp_display(character: &Character, slots: i32) -> String {
    if character.is_player() {
        bar_display(
            slots,
            character.xp,
            character.xp_for_next(),
            "cyan",
            "bright black",
        )
    } else {
        // enemies don't have experience
        String::new()
    }
}

fn bar_display(
    slots: i32,
    current: i32,
    total: i32,
    current_color: &str,
    missing_color: &str,
) -> String {
    let (filled, rest) = bar_slots(slots, total, current);
    let current = (0..filled)
        .map(|_| "x")
        .collect::<String>()
        .color(current_color);
    let missing = (0..rest)
        .map(|_| "-")
        .collect::<String>()
        .color(missing_color);
    format!("[{}{}]", current, missing)
}

fn bar_slots(slots: i32, total: i32, current: i32) -> (i32, i32) {
    let units = (current as f64 * slots as f64 / total as f64).ceil() as i32;
    (units, slots - units)
}

fn format_gold(gold: i32) -> ColoredString {
    format!("{}g", gold).yellow()
}

fn format_gold_plus(gold: i32) -> ColoredString {
    format!("+{}g", gold).yellow()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_slots() {
        // simple case 1:1 between points and slots
        let slots = 4;
        let total = 4;
        assert_eq!((0, 4), bar_slots(slots, total, 0));
        assert_eq!((1, 3), bar_slots(slots, total, 1));
        assert_eq!((2, 2), bar_slots(slots, total, 2));
        assert_eq!((3, 1), bar_slots(slots, total, 3));
        assert_eq!((4, 0), bar_slots(slots, total, 4));

        let total = 10;
        assert_eq!((0, 4), bar_slots(slots, total, 0));
        assert_eq!((1, 3), bar_slots(slots, total, 1));
        assert_eq!((1, 3), bar_slots(slots, total, 2));
        assert_eq!((2, 2), bar_slots(slots, total, 3));
        assert_eq!((2, 2), bar_slots(slots, total, 4));
        assert_eq!((2, 2), bar_slots(slots, total, 5));
        assert_eq!((3, 1), bar_slots(slots, total, 6));
        assert_eq!((3, 1), bar_slots(slots, total, 7));
        // this one I would maybe like to show as 3, 1
        assert_eq!((4, 0), bar_slots(slots, total, 8));
        assert_eq!((4, 0), bar_slots(slots, total, 9));
        assert_eq!((4, 0), bar_slots(slots, total, 10));
    }
}
