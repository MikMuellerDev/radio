use crate::audio::Error as AudioError;
use actix_files::NamedFile;
use actix_identity::Identity;
use actix_web::{
    get, post,
    web::{Data, Json},
    Error, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};

use crate::State;

#[derive(Deserialize)]
pub(crate) struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub(crate) struct PlayRequest {
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

#[get("/logout")]
pub(crate) async fn logout(user: Identity) -> Result<HttpResponse, Error> {
    debug!("user `{}` is logging out", user.id().unwrap());
    Identity::logout(user);
    Ok(HttpResponse::Found()
        .append_header(("Location", "/login"))
        .finish())
}

#[post("/api/login")]
pub(crate) async fn post_login(
    request: HttpRequest,
    user: Option<Identity>,
    body: Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    match user {
        Some(identity) => {
            debug!(
                "user `{}` is already logged in",
                identity.id().expect("the user always has a valid id")
            );
            return Ok(HttpResponse::Ok().json(Response::ok("already logged in")));
        }
        None => debug!("logging in: no user logged in"),
    };

    // validate credentials here
    if body.username == "admin" && body.password == "pw" {
        Identity::login(&request.extensions(), body.username.clone()).unwrap();
        Ok(HttpResponse::Ok().json(Response::ok("successfully logged in")))
    } else {
        Ok(HttpResponse::Forbidden()
            .json(Response::err("login failed", "bad credentials".to_string())))
    }
}

#[get("/api/stations")]
pub(crate) async fn get_stations(data: Data<State>) -> HttpResponse {
    HttpResponse::Ok().json(&data.config.stations)
}

#[post("/api/play")]
pub(crate) async fn post_play(
    data: Data<State>,
    request: Json<PlayRequest>,
    _user: Identity,
) -> HttpResponse {
    let mut player = data.player.lock().await;

    match player.play(request.url.clone()).await {
        Ok(_) => HttpResponse::Ok().json(Response::ok("started playback")),
        Err(err) => HttpResponse::ServiceUnavailable()
            .json(Response::err("could not start playback", err.to_string())),
    }
}

#[post("/api/stop")]
pub(crate) async fn post_stop(data: Data<State>, _user: Identity) -> impl Responder {
    let mut player = data.player.lock().await;
    match player.stop() {
        Ok(_) => HttpResponse::Ok().json(Response::ok("stopped playing")),
        Err(err @ AudioError::NotPlaying) => HttpResponse::BadRequest()
            .json(Response::err("could not stop the player", err.to_string())),
        Err(err) => HttpResponse::ServiceUnavailable()
            .json(Response::err("could not stop the player", err.to_string())),
    }
}

#[get("/")]
pub(crate) async fn get_dash(
    user: Option<Identity>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    match user {
        Some(_) => Ok(NamedFile::open("./radio-web/dist/html/dash.html")?.into_response(&req)),
        None => Ok(HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish()),
    }
}

#[get("/settings")]
pub(crate) async fn get_settings(
    user: Option<Identity>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    match user {
        Some(_) => Ok(NamedFile::open("./radio-web/dist/html/settings.html")?.into_response(&req)),
        None => Ok(HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish()),
    }
}

#[get("/login")]
pub(crate) async fn get_login(
    user: Option<Identity>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    match user {
        Some(_) => Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish()),
        None => Ok(NamedFile::open("./radio-web/dist/html/login.html")?.into_response(&req)),
    }
}
