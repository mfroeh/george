use std::fs;

use log::{error, info, warn};

use crate::{pretty_path, Mapping, cache::Cache};

// TODO: Add option to remove now empty folders
pub fn clean(mut cache: Cache) -> Cache {
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
        } else {
            error!("{}: failed to remove", mapping);
            not_removed.push(mapping);
        }
    }

    Cache::new(not_removed)
}
