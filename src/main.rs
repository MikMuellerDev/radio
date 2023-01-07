use std::path::PathBuf;

use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web::Data, App, HttpServer};
use anyhow::Context;
use config::Config;
use env_logger::Env;
use tokio::sync::Mutex;

mod audio;
mod config;
mod decoder;
mod routes;

use crate::audio::Player;

#[macro_use]
extern crate log;

pub(crate) struct State {
    curr_station_id: Option<String>,
    player: Mutex<Player>,
    config: Config,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config_path = "./config.toml";
    let config = match config::read(&PathBuf::from(config_path))
        .with_context(|| format!("could not read or create config file at `{config_path}`"))?
    {
        Some(config) => {
            info!("Found existing config file at `{config_path}`");
            config
        }
        None => {
            info!("Created a new configuration file at `{config_path}`");
            warn!("Canceling startup due to missing configuration");
            return Ok(());
        }
    };

    let key = Key::from(config.session_key.as_bytes());
    let port = config.port;

    let data = Data::new(State {
        curr_station_id: None,
        player: Mutex::new(Player::new()?),
        config,
    });

    HttpServer::new(move || {
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                key.clone(),
            ))
            .app_data(data.clone())
            .service(Files::new("/assets", "./radio-web/dist/assets/"))
            .service(Files::new("/images", "./images"))
            // HTML endpoints
            .service(routes::get_dash)
            .service(routes::get_settings)
            .service(routes::get_login)
            // API endpoints
            .service(routes::post_login)
            .service(routes::logout)
            .service(routes::get_stations)
            .service(routes::post_play)
            .service(routes::post_stop)
    })
    .bind(("0.0.0.0", port))
    .with_context(|| "could not start webserver")?
    .run()
    .await
    .with_context(|| "cold not start webserver")?;

    Ok(())
}
