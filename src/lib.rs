use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use path_absolutize::*;
use serde::{Deserialize, Serialize};

pub mod cache;
pub mod clean;
pub mod config;
pub mod deploy;

pub static HOME_DIR: Lazy<Option<String>> = Lazy::new(|| {
    if let Ok(cow) = shellexpand::env("$HOME") {
        Some(cow.to_string())
    } else {
        None
    }
});

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Mapping {
    /// The name of the link (i.e. the destination)
    name: PathBuf,
    /// The target that will be pointed to
    target: PathBuf,
}

impl Display for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{} -> {}]",
            pretty_path(&self.name),
            pretty_path(&self.target)
        )
    }
}

impl Mapping {
    pub fn new(name: &str, target: &str) -> Mapping {
        let name = shellexpand::tilde(name).to_string();
        let target = shellexpand::tilde(target).to_string();

        let name = PathBuf::from(name).absolutize().unwrap().into();
        let target = PathBuf::from(target).absolutize().unwrap().into();
        Mapping { name, target }
    }

    pub fn name(&self) -> &Path {
        &self.name
    }

    pub fn destination(&self) -> &Path {
        self.name()
    }

    pub fn target(&self) -> &Path {
        &self.target
    }
}

pub fn pretty_path(path: &Path) -> String {
    let str = path.to_str().unwrap();
    if let Some(home) = &*HOME_DIR {
        str.replacen(home, "~", 1)
    } else {
        str.to_owned()
    }
}
