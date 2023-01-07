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

#[derive(Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub image_file: PathBuf,
}

pub fn read(path: &Path) -> Result<Option<Config>> {
    match path.exists() {
        true => {
            let raw_config = fs::read_to_string(path)?;
            let config = toml::from_str::<Config>(&raw_config)?;
            let key_len = config.session_key.len();
            if key_len < 64 {
                bail!("insufficient key length: key must be >= 64 characters, found {key_len}",)
            }
            if config.stations.is_empty() {
                bail!("no stations configured: there must be at least one stations")
            }

            let mut station_ids = HashSet::new();
            for station in &config.stations {
                if !station_ids.insert(&station.id) {
                    bail!("duplicate station ID `{} ", station.id)
                }
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
            Ok(Some(config))
        }
        false => {
            // create the config file using a default
            fs::create_dir_all(path.parent().unwrap())?;
            let mut file = File::create(path)?;
            /* let config = Config {
                           port: 8083,
                           session_key: "my_secret_session_key_over_64_characters".to_string(),
                           users: vec![User {
                               username: "admin".to_string(),
                               password: "secret".to_string(),
                           }],
                           stations: vec![Station {
                               url: "https://example.com/stream".to_string(),
                               name: "Example Radio".to_string(),
                               image_path: PathBuf::from("/opt/radio/assets/example.png"),
                           }],
                       };
                       file.write_all(toml::to_string(&config).unwrap().as_bytes());
            */
            file.write_all(include_bytes!("default_config.toml"))?;
            Ok(None)
        }
    }
}
