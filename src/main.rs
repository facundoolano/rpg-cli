use game::Game;

mod character;
mod game;
mod location;
mod log;

use crate::location::Location;

fn main() {
    let mut game = Game::load().unwrap_or_else(|_| Game::new());

    // FIXME temporary test
    match std::env::args().nth(1) {
        Some(arg) if arg == "--pwd" => {
            println!("{}", game.location.path.to_string_lossy());
        }
        Some(dest) => {
            let dest = Location::from(&dest);

            match game.walk_towards(&dest) {
                Err(game::Error::GameOver) => game.reset(),
                _ => game.save().unwrap(),
            }
        }
        _ => {
            log::status(&game.player, &game.location);
        }
    }
}
