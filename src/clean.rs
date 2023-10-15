use std::fs;

use log::{error, info, warn};

use crate::{cache::Cache, pretty_path, Mapping};

#[derive(Debug, Default)]
pub struct CleanOptions {
    rmdir: bool,
}

impl CleanOptions {
    pub fn new(rmdir: bool) -> Self {
        CleanOptions { rmdir }
    }
}

pub fn clean(mut cache: Cache, opt: CleanOptions) -> Cache {
    let mut not_removed = Vec::new();

    for mapping in cache.mappings.take().unwrap().into_iter() {
        let Mapping { name, target } = &mapping;
        if !mapping.name().exists() {
            info!(
                "{}: {} doesn't exist anymore, skipping",
                mapping,
                pretty_path(name)
            );
            continue;
        }

        let target_cur = mapping.name().canonicalize().unwrap();
        if target_cur != target.canonicalize().unwrap() {
            warn!(
                "{}: {} points to {} now, won't remove",
                mapping,
                pretty_path(name),
                pretty_path(&target_cur)
            );
            not_removed.push(mapping);
            continue;
        }

        if let Ok(()) = fs::remove_file(name) {
            info!("{}: removed", mapping);
            if let Some(parent) = name.parent() {
                if opt.rmdir && parent.read_dir().unwrap().next().is_none() {
                    if fs::remove_dir(parent).is_ok() {
                        info!(
                            "{}: removed empty parent dir {}",
                            mapping,
                            pretty_path(parent)
                        );
                    } else {
                        error!(
                            "{}: failed to remove empty parent dir {}",
                            mapping,
                            pretty_path(parent)
                        );
                    }
                }
            }
        } else {
            error!("{}: failed to remove", mapping);
            not_removed.push(mapping);
        }
    }

    Cache::new(not_removed)
}
