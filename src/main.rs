use game::Game;

mod game;
mod player;

fn main() {
    // TODO maybe separate new/save from load?
    let game = Game::load().unwrap();
    game.save().unwrap();
    player_status(&game);
}

fn player_status(game: &Game) {
    let player = &game.player;
    println!("{}@{} ", player.name, game.location.display());
    println!(
        "lv:{} hp:{}/{} xp:{}",
        player.level, player.current_hp, player.max_hp, player.xp
    );
}
