use std::path::PathBuf;

use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},
    web::Data,
    App, HttpServer,
};
use anyhow::Context;
use clap::Parser;
use config::Config;
use env_logger::Env;
use settings::Settings;
use tokio::sync::Mutex;

mod audio;
mod cli;
mod config;
mod decoder;
mod routes;
mod settings;

use crate::{
    audio::Player,
    cli::{Args, Command},
};

#[macro_use]
extern crate log;

pub(crate) struct State {
    player: Mutex<Player>,
    config: Config,
    settings: Mutex<Settings>,
}

const CONFIG_PATH: &str = "./config.toml";
const SETTINGS_PATH: &str = "./settings.toml";

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config_path = match args.config_path {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(CONFIG_PATH),
    };

    let settings = settings::read(&PathBuf::from(SETTINGS_PATH))
        .with_context(|| format!("could not read or create settings file at `{SETTINGS_PATH}`"))?;

    let config = match config::read(&config_path).with_context(|| {
        format!(
            "could not read or create config file at `{}`",
            config_path.to_string_lossy()
        )
    })? {
        Some(config) => {
            info!(
                "Found existing config file at `{}`",
                config_path.to_string_lossy()
            );
            config
        }
        None => {
            info!(
                "Created a new configuration file at `{}`",
                config_path.to_string_lossy()
            );
            if args.subcommand == Command::Files {
                warn!("Canceling startup due to missing configuration");
            }
            return Ok(());
        }
    };

    if args.subcommand != Command::Run {
        return Ok(());
    }

    let key = Key::from(config.session_key.as_bytes());
    let port = config.port;

    let mut player = Player::new(settings.volume_percent, settings.alsa_device_index)?;

    match config.stations.iter().find(|s| s.auto_start) {
        Some(station) => {
            info!("Starting auto start stream with ID `{}`...", station.id);
            player
                .play(station.clone())
                .await
                .with_context(|| "could not start auto start stream")?;
            info!("Successfully started stream `{}` as it is marked as auto start", station.id);
        }
        None => {
            debug!("No stream is marked as auto start, continuing normal startup...")
        }
    }

    let data = Data::new(State {
        player: Mutex::new(player),
        config,
        settings: Mutex::new(settings),
    });

    let server = HttpServer::new(move || {
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(false) // required in order to allow HTTP sessions
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(7)))
                    .build(),
            )
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
