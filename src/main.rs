use game::Game;

mod character;
mod game;
mod location;

use crate::location::Location;

fn main() {
    let mut game = Game::load().unwrap_or_else(|_| Game::new());

    if let Some(dest) = std::env::args().nth(1) {
        let dest = Location::from(&dest);

        match game.walk_towards(&dest) {
            Err(game::Error::GameOver) => game.reset(),
            _ => game.save().unwrap(),
        }
    }

    player_status(&game);
}

// FIXME most of this belongs in player struct
fn player_status(game: &Game) {
    let player = &game.player;
    println!("{}@{}", player, game.location);
    println!("  hp:{}", player.hp_display());
    println!("  xp:{}/{}", player.xp, player.xp_for_next());
}
