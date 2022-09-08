use std::{env, process, time::Duration};
use std::thread;

use audio::Player;

mod audio;

#[tokio::main]
async fn main() {
    let player = Player::new().unwrap_or_else(|err| {
        eprintln!("Error occured: {err}");
        process::exit(1);
    });
    player
        .play(&env::var("URL").unwrap_or_else(|_| "invalid".to_string()))
        .await
        .unwrap_or_else(|err| {
            eprintln!("Error occured: {err}");
            process::exit(1);
        });
    thread::sleep(Duration::from_secs(5));
}
