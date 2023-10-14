use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use crate::DeployResult;

#[derive(Serialize, Deserialize)]
pub struct CreatedLinkCache {
    links: Vec<PathBuf>,
}

impl CreatedLinkCache {
    pub fn new(result: DeployResult) -> CreatedLinkCache {
        let links = match result {
            DeployResult::Some { created } => created,
            DeployResult::Some { created } => created,
            DeployResult::None => Vec::new(),
        };
        CreatedLinkCache { links }
    }
}