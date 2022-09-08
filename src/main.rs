use std::process;

use audio::Player;

mod audio;

#[tokio::main]
async fn main() {
    let player = Player::new().unwrap_or_else(|err| {
        eprintln!("Error occured: {err}");
        process::exit(1);
    });
    player.play(url, 64_000).await.unwrap_or_else(|err| {
        eprintln!("Error occured: {err}");
        process::exit(1);
    });
}
