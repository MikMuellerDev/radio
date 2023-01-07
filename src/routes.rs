use std::path::PathBuf;

use crate::{
    audio::{self, Error as AudioError},
    SETTINGS_PATH,
};
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
pub(crate) struct LoginReq {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub(crate) struct PlayReq {
    #[serde(rename = "stationId")]
    station_id: String,
}

#[derive(Deserialize)]
pub(crate) struct VolumeReq {
    volume: u8,
}

#[derive(Serialize)]
pub(crate) struct StatusRes {
    #[serde(rename = "stationId")]
    station_id: Option<String>,
    volume: u8,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct DeviceReqRes {
    index: usize,
}

#[derive(Serialize)]
struct GenericResponse {
    message: &'static str,
    error: Option<String>,
}

impl GenericResponse {
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
    body: Json<LoginReq>,
) -> Result<HttpResponse, Error> {
    match user {
        Some(identity) => {
            debug!(
                "user `{}` is already logged in",
                identity.id().expect("the user always has a valid id")
            );
            return Ok(HttpResponse::Ok().json(GenericResponse::ok("already logged in")));
        }
        None => debug!("logging in: no user logged in"),
    };

    // validate credentials here
    if body.username == "admin" && body.password == "pw" {
        Identity::login(&request.extensions(), body.username.clone()).unwrap();
        Ok(HttpResponse::Ok().json(GenericResponse::ok("successfully logged in")))
    } else {
        Ok(HttpResponse::Forbidden().json(GenericResponse::err(
            "login failed",
            "bad credentials".to_string(),
        )))
    }
}

#[get("/api/status")]
pub(crate) async fn get_status(data: Data<State>) -> HttpResponse {
    let mut player = data.player.lock().await;
    let settings = data.settings.lock().await;
    let station_id = player.curr_station_id();

    HttpResponse::Ok().json(StatusRes {
        station_id,
        volume: settings.volume_percent,
    })
}

#[get("/api/stations")]
pub(crate) async fn get_stations(data: Data<State>) -> HttpResponse {
    HttpResponse::Ok().json(&data.config.stations)
}

#[post("/api/play")]
pub(crate) async fn post_play(
    data: Data<State>,
    request: Json<PlayReq>,
    _user: Identity,
) -> HttpResponse {
    let mut player = data.player.lock().await;

    match data
        .config
        .stations
        .iter()
        .find(|s| s.id == request.station_id)
    {
        Some(station) => match player.play(station.clone()).await {
            Ok(_) => HttpResponse::Ok().json(GenericResponse::ok("started playback")),
            Err(err) => HttpResponse::ServiceUnavailable().json(GenericResponse::err(
                "could not start playback",
                err.to_string(),
            )),
        },
        None => HttpResponse::UnprocessableEntity().json(GenericResponse::err(
            "could not start playback",
            "this station ID does not exist".to_string(),
        )),
    }
}

#[post("/api/stop")]
pub(crate) async fn post_stop(data: Data<State>, _user: Identity) -> impl Responder {
    let mut player = data.player.lock().await;

    match player.stop(true) {
        Ok(_) => HttpResponse::Ok().json(GenericResponse::ok("stopped playing")),
        Err(err @ AudioError::NotPlaying) => HttpResponse::BadRequest().json(GenericResponse::err(
            "could not stop the player",
            err.to_string(),
        )),
        Err(err) => HttpResponse::ServiceUnavailable().json(GenericResponse::err(
            "could not stop the player",
            err.to_string(),
        )),
    }
}

#[post("/api/volume")]
pub(crate) async fn post_volume(
    data: Data<State>,
    request: Json<VolumeReq>,
    _user: Identity,
) -> impl Responder {
    let mut player = data.player.lock().await;
    player.set_volume(request.volume);

    let settings = &mut data.settings.lock().await;
    settings.volume_percent = request.volume;

    match settings.write(&PathBuf::from(SETTINGS_PATH)) {
        Ok(_) => HttpResponse::Ok().json(GenericResponse::ok("successfully set volume")),
        Err(err) => {
            error!("Could not write to settings file at `{SETTINGS_PATH}`: {err}");
            HttpResponse::InternalServerError().json(GenericResponse::err(
                "could not set volume",
                "could not write to settings file".to_string(),
            ))
        }
    }
}

#[get("/api/devices")]
pub(crate) async fn get_devices(_user: Identity) -> impl Responder {
    match audio::list_host_devices() {
        Ok(devices) => HttpResponse::Ok().json(devices),
        Err(err) => HttpResponse::ServiceUnavailable().json(GenericResponse::err(
            "could not list devices",
            err.to_string(),
        )),
    }
}

#[get("/api/device")]
pub(crate) async fn get_device(data: Data<State>, _user: Identity) -> impl Responder {
    let settings = data.settings.lock().await;
    HttpResponse::Ok().json(DeviceReqRes {
        index: settings.alsa_device_index,
    })
}

#[post("/api/device")]
pub(crate) async fn post_device(
    data: Data<State>,
    request: Json<DeviceReqRes>,
    _user: Identity,
) -> impl Responder {
    match audio::list_host_devices() {
        Ok(devices) => {
            if devices.get(request.index).is_none() {
                return HttpResponse::BadRequest().json(GenericResponse::err(
                    "could not change output device",
                    "this device does not exist".to_string(),
                ));
            }

            let settings = &mut data.settings.lock().await;
            settings.alsa_device_index = request.index;

            if let Err(err) = settings.write(&PathBuf::from(SETTINGS_PATH)) {
                error!("Could not write to settings file at `{SETTINGS_PATH}`: {err}");
                return HttpResponse::InternalServerError().json(GenericResponse::err(
                    "could not change output device",
                    "could not write to settings file".to_string(),
                ));
            }

            let player = &mut data.player.lock().await;
            match player.set_output_device(request.index).await {
                Ok(_) => HttpResponse::Ok()
                    .json(GenericResponse::ok("successfully changed output device")),
                Err(err) => HttpResponse::ServiceUnavailable().json(GenericResponse::err(
                    "could not restart player",
                    err.to_string(),
                )),
            }
        }
        Err(err) => HttpResponse::ServiceUnavailable().json(GenericResponse::err(
            "could not list devices",
            err.to_string(),
        )),
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
