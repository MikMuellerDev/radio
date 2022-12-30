use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Context;
use audio::Error;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// TODO: place under `audio` module
mod audio;
mod decoder;

use crate::audio::Player;

#[macro_use]
extern crate log;

struct State {
    player: Mutex<Player>,
}

#[derive(Deserialize)]
struct PlayRequest {
    url: String,
}

#[derive(Serialize)]
struct Response {
    message: &'static str,
    error: Option<String>,
}

impl Response {
    fn err(message: &'static str, error: String) -> Self {
        Self {
            message,
            error: Some(error),
        }
    }

    fn ok(message: &'static str) -> Self {
        Self {
            message,
            error: None,
        }
    }
}

#[post("/play")]
async fn play(data: web::Data<State>, request: web::Json<PlayRequest>) -> impl Responder {
    let mut player = data.player.lock().await;

    match player.play(request.url.clone()).await {
        Ok(_) => HttpResponse::Ok().json(Response::ok("started playback")),
        Err(err) => HttpResponse::ServiceUnavailable()
            .json(Response::err("could not start playback", err.to_string())),
    }
}

#[post("/stop")]
async fn stop(data: web::Data<State>) -> impl Responder {
    let mut player = data.player.lock().await;
    match player.stop() {
        Ok(_) => HttpResponse::Ok().json(Response::ok("stopped playing")),
        Err(err @ Error::NotPlaying) => HttpResponse::BadRequest()
            .json(Response::err("could not stop the player", err.to_string())),
        Err(err) => HttpResponse::ServiceUnavailable()
            .json(Response::err("could not stop the player", err.to_string())),
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let data = web::Data::new(State {
        player: Mutex::new(Player::new()?),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(play)
            .service(stop)
    })
    .bind(("0.0.0.0", 8080))
    .with_context(|| "cold not start webserver")?
    .run()
    .await
    .with_context(|| "cold not start webserver")?;

    Ok(())
}
