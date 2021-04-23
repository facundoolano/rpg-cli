use player::Player;

mod player;

fn main() {
    let player = Player::new();
    player_status(&player);
}

fn player_status(player: &Player) {
    println!("{} lv {}", player.name, player.level);
    println!("hp {}/{}", player.current_hp, player.max_hp);
    println!("xp {} next level: TODO", player.xp);
}
