use std::{error::Error, fs, io, path::PathBuf};

use chrono::{Local, format::format};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{Mapping, clean, HOME_DIR};

#[derive(Serialize, Deserialize, Default)]
pub struct Cache {
    /// The mappings that existed after the last deploy
    pub (in super) mappings: Option<Vec<Mapping>>,
}

impl Cache {
    pub fn new(existing: Vec<Mapping>) -> Cache {
        Cache {
            mappings: Some(existing),
        }
    }

    pub fn load() -> Result<Cache, Box<dyn Error>> {
        let cache_home = if let Ok(cache_home) = shellexpand::env("$XDG_CACHE_HOME/george") {
            PathBuf::from(cache_home.into_owned())
        } else if let Some(home) = &*HOME_DIR {
            PathBuf::from(format!("{home}/.cache/george"))
        } else {
            // TODO: Cache error?
            return Err("Failed to expand both $HOME and $XDG_CACHE_HOME, cannot find cache".into());
        };

        if let Some(e) = WalkDir::new(cache_home)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|f| f.path().is_file())
        {
            let contents = fs::read_to_string(e.path())?;
            Ok(toml::from_str::<Cache>(&contents)?)
        } else {
            Ok(Cache::default())
        }
    }

    pub fn save(self) -> Result<(), Box<dyn Error>> {
        let cache_home = if let Ok(cache_home) = shellexpand::env("$XDG_CACHE_HOME/george") {
            PathBuf::from(cache_home.into_owned())
        } else if let Some(home) = &*HOME_DIR {
            PathBuf::from(format!("{home}/.cache/george"))
        } else {
            // TODO: Cache error?
            return Err("Failed to expand both $HOME and $XDG_CACHE_HOME, cannot find cache".into());
        };

        if !cache_home.exists() {
            fs::create_dir_all(&cache_home)?;
        }

        let filename = Local::now().to_string();

        let toml = toml::to_string_pretty(&self)?;
        Ok(fs::write(cache_home.join(filename), toml)?)
    }

    pub fn contains(&self, mapping: &Mapping) -> bool {
        if let Some(mappings) = &self.mappings {
            mappings.contains(mapping)
        } else {
            false
        }
    }

    pub fn mappings(&self) -> &[Mapping] {
        if let Some(mappings) = &self.mappings {
            mappings
        } else {
            &[]
        }
    }
}
