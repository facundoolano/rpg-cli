use game::Game;

mod game;
mod location;
mod player;

use crate::location::Location;

fn main() {
    // FIXME don't assume arguments, properly parse them
    let destination = std::env::args().nth(1).unwrap();
    let location = Location::from(&destination);
    println!("{:?}", location.is_home());

    // TODO maybe separate new/save from load?
    let game = Game::load().unwrap();
    game.save().unwrap();
    player_status(&game);
}

fn player_status(game: &Game) {
    let player = &game.player;
    println!("{}@{} ", player.name, game.location);
    println!(
        "lv:{} hp:{}/{} xp:{}",
        player.level, player.current_hp, player.max_hp, player.xp
    );
}
