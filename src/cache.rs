use std::{
    error::Error,
    fs::{self},
    path::PathBuf,
};

use chrono::Local;
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use crate::{Mapping, HOME_DIR};

#[derive(Serialize, Deserialize)]
pub struct Cache {
    /// The mappings that existed after the last deploy
    pub(super) mappings: Option<Vec<Mapping>>,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            mappings: Some(Vec::new()),
        }
    }
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
            return Err(
                "Failed to expand both $HOME and $XDG_CACHE_HOME, cannot find cache".into(),
            );
        };

        let sort = |lhs: &DirEntry, rhs: &DirEntry| -> std::cmp::Ordering {
            lhs.file_name().cmp(rhs.file_name()).reverse()
        };

        if let Some(e) = WalkDir::new(cache_home)
            .sort_by(sort)
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
            return Err(
                "Failed to expand both $HOME and $XDG_CACHE_HOME, cannot find cache".into(),
            );
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

#[cfg(test)]
mod tests {
    use std::env;

    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn no_cache_no_cache_home() {
        let cache_home = env::current_dir().unwrap().join("cache");
        if cache_home.exists() {
            fs::remove_dir_all(&cache_home).unwrap();
        }

        env::remove_var("XDG_CACHE_HOME");
        env::remove_var("HOME");
        env::set_var("HOME", cache_home.to_str().unwrap());
        let cache = Cache::load().unwrap_or_default();
        assert!(cache.save().is_ok());
        assert!(cache_home.exists());
        assert!(cache_home.read_dir().unwrap().next().is_some());
        fs::remove_dir_all(&cache_home).unwrap();

        env::remove_var("HOME");
        env::set_var("XDG_CACHE_HOME", cache_home.to_str().unwrap());
        let cache = Cache::load().unwrap_or_default();
        assert!(cache.save().is_ok());
        assert!(cache_home.exists());
        assert!(cache_home.read_dir().unwrap().next().is_some());
        fs::remove_dir_all(&cache_home).unwrap();
    }
}
