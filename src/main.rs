use game::Game;

mod game;
mod location;
mod player;

use crate::location::Location;

fn main() {
    // FIXME don't assume arguments, properly parse them
    let dest = std::env::args().nth(1).unwrap();
    let dest = Location::from(&dest);

    // TODO maybe separate new/save from load?
    let mut game = Game::load().unwrap();

    game.walk_towards(&dest);
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
