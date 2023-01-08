use anyhow::Result;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::audio;

#[derive(Serialize, Deserialize)]
pub(crate) struct Settings {
    pub(crate) alsa_device_index: usize,
    pub(crate) volume_percent: u8,
}

impl Settings {
    fn default() -> Result<Self> {
        let alsa_device_idx = audio::default_device_idx()?;
        Ok(Self {
            alsa_device_index: alsa_device_idx,
            volume_percent: 100,
        })
    }

    pub(crate) fn write(&self, path: &Path) -> Result<()> {
        match self.write_to_file(path) {
            Ok(_) => {
                trace!("Successfully written to settings file");
                Ok(())
            }
            Err(err) => {
                error!(
                    "Could not write to settings file at `{}`: {err}",
                    path.to_string_lossy()
                );
                Err(err)
            }
        }
    }

    fn write_to_file(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;
        file.write_all(serde_json::to_string_pretty(self).unwrap().as_bytes())?;
        Ok(())
    }
}

pub(crate) fn read(path: &Path) -> Result<Settings> {
    match path.exists() {
        true => {
            let raw_settings = fs::read_to_string(path)?;
            let config = serde_json::from_str::<Settings>(&raw_settings)?;
            Ok(config)
        }
        false => {
            // create the config file using a default
            fs::create_dir_all(path.parent().unwrap())?;
            let settings = Settings::default()?;
            let mut file = File::create(path)?;
            file.write_all(serde_json::to_string_pretty(&settings).unwrap().as_bytes())?;
            Ok(settings)
        }
    }
}
