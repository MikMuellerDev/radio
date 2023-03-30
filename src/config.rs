use anyhow::{bail, Ok, Result};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub session_key: String,
    pub users: Vec<User>,
    pub stations: Vec<Station>,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub image_file: PathBuf,
    pub auto_restart: bool,
    pub auto_start: bool,
}

impl Config {
    fn validate(&self) -> Result<()> {
        let key_len = self.session_key.len();
        if key_len < 64 {
            bail!("insufficient key length: key must be >= 64 characters, found {key_len}",)
        }
        if self.stations.is_empty() {
            bail!("no stations configured: there must be at least one stations")
        }

        let mut station_ids = HashSet::new();
        let mut auto_start_id = None;

        for station in &self.stations {
            // check if station ID is `url`
            if station.id == "url" {
                bail!("station ID cannot be `url`")
            }

            // check if station ID is unique
            if !station_ids.insert(&station.id) {
                bail!("duplicate station ID `{} ", station.id)
            }

            if station.auto_start {
                // validate that only one station is marked as `auto_start`
                match auto_start_id {
                Some(old) => bail!("station `{}` cannot be `auto_start`: the station `{old}` is already marked as `auto_start`", station.id),
                None =>  auto_start_id = Some(station.id.to_string()),
            }
            }

            // validate image of the station
            let path = PathBuf::from("./images").join(&station.image_file);
            if !path.exists() {
                bail!(
                    "station `{}` has an invalid image name: `{}`: path does not exist",
                    station.name,
                    station.image_file.to_string_lossy()
                )
            }
            if !path.is_file() {
                bail!(
                    "station `{}` has an invalid image path: `{}`: path is not a file",
                    station.name,
                    station.image_file.to_string_lossy()
                )
            }
        }
        Ok(())
    }
}

pub fn read(path: &Path) -> Result<Option<Config>> {
    match path.exists() {
        true => {
            let raw_config = fs::read_to_string(path)?;
            let config = toml::from_str::<Config>(&raw_config)?;
            config.validate()?;
            Ok(Some(config))
        }
        false => {
            // create the config file using a default
            fs::create_dir_all(path.parent().unwrap())?;
            let mut file = File::create(path)?;

            /* let config = Config {
                port: 8083,
                session_key: "my_secret_session_key_which_must_be_over_64_characters_in_length".to_string(),
                users: vec![User {
                    username: "admin".to_string(),
                    password: "secret".to_string(),
                }],
                stations: vec![Station {
                    id: "example".to_string(),
                    name: "Example Radio".to_string(),
                    description: "This is an example radio".to_string(),
                    url: "https://example.com/stream".to_string(),
                    image_file: PathBuf::from("example.png"),
                    auto_restart: true,
                }],
            };
            file.write_all(toml::to_string(&config).unwrap().as_bytes()); */

            file.write_all(include_bytes!("default_config.toml"))?;
            Ok(None)
        }
    }
}
