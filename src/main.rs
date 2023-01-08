use std::path::PathBuf;

use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web::Data, App, HttpServer};
use anyhow::Context;
use config::Config;
use env_logger::Env;
use settings::Settings;
use tokio::sync::Mutex;

mod audio;
mod config;
mod decoder;
mod routes;
mod settings;

use crate::audio::Player;

#[macro_use]
extern crate log;

pub(crate) struct State {
    player: Mutex<Player>,
    config: Config,
    settings: Mutex<Settings>,
}

const CONFIG_PATH: &str = "./config.toml";
const SETTINGS_PATH: &str = "./settings.json";

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = match config::read(&PathBuf::from(CONFIG_PATH))
        .with_context(|| format!("could not read or create config file at `{CONFIG_PATH}`"))?
    {
        Some(config) => {
            info!("Found existing config file at `{CONFIG_PATH}`");
            config
        }
        None => {
            info!("Created a new configuration file at `{CONFIG_PATH}`");
            warn!("Canceling startup due to missing configuration");
            return Ok(());
        }
    };

    let settings = settings::read(&PathBuf::from(SETTINGS_PATH))
        .with_context(|| format!("could not read or create settings file at `{SETTINGS_PATH}`"))?;

    let key = Key::from(config.session_key.as_bytes());
    let port = config.port;

    let data = Data::new(State {
        player: Mutex::new(Player::new(
            settings.volume_percent,
            settings.alsa_device_index,
        )?),
        config,
        settings: Mutex::new(settings),
    });

    let server = HttpServer::new(move || {
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
            .service(routes::get_status)
            .service(routes::get_stations)
            .service(routes::post_play)
            .service(routes::post_stop)
            .service(routes::post_volume)
            .service(routes::get_devices)
            .service(routes::post_device)
            .service(routes::get_device)
    })
    .bind(("::0", port))
    .with_context(|| "could not start webserver")?;

    info!("Radio is running on `http://localhost:{}`", port);

    server
        .run()
        .await
        .with_context(|| "cold not start webserver")?;

    info!("Radio is shutting down...");

    Ok(())
}
