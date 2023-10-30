use std::fs::{self};


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

        if let Ok(link_target) = fs::read_link(name) {
            // If it doesn't exist, we want to remove without checking this as it would panic
            if link_target.exists() {
                let link_target = link_target.canonicalize().unwrap();
                if link_target != target.canonicalize().unwrap() {
                    warn!(
                        "{}: {} points to {} now, treating as removed",
                        mapping,
                        pretty_path(name),
                        pretty_path(&link_target)
                    );
                    continue;
                }
            }
        } else {
            warn!(
                "{}: {} doesn't exist anymore or is not a symbolic link, treating as removed",
                mapping,
                pretty_path(name)
            );
            continue;
        };

        if let Ok(()) = fs::remove_file(name) {
            info!("{}: removed", mapping);
            if !opt.rmdir {
                continue;
            }

            let mut cur = name.as_path();
            while let Some(parent) = cur.parent() {
                if parent.exists() && parent.read_dir().unwrap().next().is_none() {
                    if fs::remove_dir(parent).is_ok() {
                        info!(
                            "{}: removed empty parent dir {}",
                            mapping,
                            pretty_path(parent)
                        );
                        cur = parent;
                    } else {
                        error!(
                            "{}: failed to remove empty parent dir {}",
                            mapping,
                            pretty_path(parent)
                        );
                        break;
                    }
                } else {
                    break;
                }
            }
        } else {
            error!("{}: failed to remove", mapping);
            not_removed.push(mapping);
        }
    }

    Cache::new(not_removed)
}
