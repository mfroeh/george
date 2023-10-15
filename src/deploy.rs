use std::{collections::HashSet, fs, os::unix::fs::symlink};

use log::{error, info, warn};
use walkdir::WalkDir;

use crate::{
    cache::Cache,
    clean::{self, CleanOptions},
    config::Config,
    pretty_path, Mapping,
};

#[derive(Debug, Default)]
pub struct DeployOptions {
    rmdir: bool,
}

impl DeployOptions {
    pub fn new(rmdir: bool) -> Self {
        DeployOptions { rmdir }
    }
}

pub fn deploy(cache: Cache, opt: DeployOptions, config: Config) -> Cache {
    let expanded: Vec<Mapping> = expand_mappings(&opt, config.mappings())
        .into_iter()
        .collect();

    // Remove all previously created mappings that have become redundant
    let redundant_mappings: Vec<Mapping> = cache
        .mappings()
        .iter()
        .filter(|m| !expanded.contains(m))
        .map(|m| m.to_owned())
        .collect();

    // If we couldn't remove some of the mappings, we have to keep them in the cache
    let mut existing = clean::clean(Cache::new(redundant_mappings), CleanOptions::new(opt.rmdir))
        .mappings
        .take()
        .unwrap();

    // Create the new mappings
    for mapping in expanded.into_iter() {
        let Mapping { name, target } = &mapping;

        if name.parent().is_some_and(|p| !p.exists()) {
            if let Ok(()) = fs::create_dir_all(name.parent().unwrap()) {
                info!("{}: created parent directory", mapping);
            } else {
                error!(
                    "{}: failed to create parent directory, won't create link",
                    mapping
                );
                continue;
            }
        }

        // If link already exists and is created by us
        if name.exists() && cache.contains(&mapping) {
            info!(
                "{}: '{}' exists already and was created by us, not creating new link",
                mapping,
                pretty_path(name)
            );
            existing.push(mapping);
            continue;
        }

        if let Ok(()) = symlink(target, name) {
            info!("{}: created mapping", mapping);
            existing.push(mapping);
        } else {
            error!("{}: failed to create mapping", mapping);
        }
    }

    Cache::new(existing)
}

fn expand_mappings(opt: &DeployOptions, mappings: &[Mapping]) -> HashSet<Mapping> {
    let mut set = HashSet::new();

    for mapping in mappings.iter() {
        let Mapping { name, target } = mapping;

        // Target has to exist
        if !target.exists() {
            warn!(
                "{}: '{}' does not exist, skipping",
                mapping,
                pretty_path(target)
            );
            continue;
        }

        // If target is dir, expand all files in dir first
        if target.is_dir() {
            let name_base = name.to_str().unwrap();
            let target_base = target.to_str().unwrap();
            let make_mapping = |target: walkdir::DirEntry| {
                let target = target.path().to_str().unwrap();
                let name = &target.replace(target_base, name_base);
                Mapping::new(name, target)
            };

            let files = WalkDir::new(target)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .map(make_mapping);

            info!("{}: beginning expansion", mapping);
            set.extend(expand_mappings(opt, &files.collect::<Vec<_>>()));
            continue;
        }

        if target.is_file() {
            info!("{}: expanded", mapping);
            set.insert(mapping.to_owned());
            continue;
        }

        warn!(
            "{}: was not expanded due to not being handled currently",
            mapping
        );
    }

    set
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::{
        fs,
        path::{Path, PathBuf},
        vec,
    };

    use super::*;

    const DOTFILE_DIR: &str = "test_dotfiles";
    const HOME_DIR: &str = "test_~";

    fn setup() {
        if Path::new(HOME_DIR).exists() {
            fs::remove_dir_all(HOME_DIR).unwrap();
        }
        fs::create_dir(HOME_DIR).unwrap();

        if Path::new(DOTFILE_DIR).exists() {
            fs::remove_dir_all(DOTFILE_DIR).unwrap();
        }
        fs::create_dir(DOTFILE_DIR).unwrap();
    }

    #[test]
    #[serial]
    fn expand() {
        setup();
        let nonempty_dir = format!("{DOTFILE_DIR}/config/nvim");
        fs::create_dir_all(&nonempty_dir).unwrap();
        let file1 = format!("{nonempty_dir}/init.lua");
        let file2 = format!("{nonempty_dir}/something.lua");
        fs::write(&file1, "").unwrap();
        fs::write(&file2, "").unwrap();
        fs::create_dir_all(format!("{DOTFILE_DIR}/config/empty")).unwrap();

        let config = Config::build(&format!("{HOME_DIR}/.config -> {DOTFILE_DIR}/config")).unwrap();
        let result = expand_mappings(&DeployOptions::default(), config.mappings());

        assert!(result.contains(&Mapping::new(
            &format!("{HOME_DIR}/.config/nvim/init.lua"),
            &file1
        )));
        assert!(result.contains(&Mapping::new(
            &format!("{HOME_DIR}/.config/nvim/something.lua"),
            &file2
        )));
    }

    #[test]
    #[serial]
    fn fail_link_file_exists() {
        setup();
        let target = format!("{DOTFILE_DIR}/.zshrc");
        fs::write(&target, "").unwrap();

        let name = format!("{HOME_DIR}/.zshrc");
        fs::write(&name, "").unwrap();

        let config = Config::build(&format!("{name} -> {target}")).unwrap();
        let result = deploy(Cache::default(), DeployOptions::default(), config);

        let expected = vec![];
        assert_eq!(result.mappings(), expected);
        assert!(!PathBuf::from(name).is_symlink());
    }

    #[test]
    #[serial]
    fn link_single_file() {
        setup();
        let target = format!("{DOTFILE_DIR}/.zshrc");
        fs::write(&target, "").unwrap();

        let name = format!("{HOME_DIR}/.zshrc");

        let config = Config::build(&format!("{name} -> {target}")).unwrap();
        let result = deploy(Cache::default(), DeployOptions::default(), config);

        let expected = vec![Mapping::new(&name, &target)];
        assert_eq!(result.mappings(), expected);
        assert!(PathBuf::from(&name).exists());
        assert!(PathBuf::from(&name).is_symlink());
        assert!(PathBuf::from(&name)
            .canonicalize()
            .is_ok_and(|p| p == PathBuf::from(&target).canonicalize().unwrap()));
    }

    #[test]
    #[serial]
    fn link_file_in_nested_dir() {
        setup();
        let target = format!("{DOTFILE_DIR}/nvim/init.lua");
        fs::create_dir(format!("{DOTFILE_DIR}/nvim")).unwrap();
        fs::write(&target, "").unwrap();

        let name = format!("{HOME_DIR}/.config/nvim/init.lua");

        let config = Config::build(&format!("{name} -> {target}")).unwrap();
        let result = deploy(Cache::default(), DeployOptions::default(), config);

        let expected = vec![Mapping::new(&name, &target)];
        assert_eq!(result.mappings(), expected);
        assert!(PathBuf::from(&name).exists());
        assert!(PathBuf::from(&name).is_symlink());
        assert!(PathBuf::from(&name)
            .canonicalize()
            .is_ok_and(|p| p == PathBuf::from(&target).canonicalize().unwrap()));
    }

    #[test]
    #[serial]
    fn link_empty_dir() {
        setup();
        let target = format!("{DOTFILE_DIR}/nvim");
        fs::create_dir(&target).unwrap();

        let name = format!("{HOME_DIR}/.config/nvim");

        let config = Config::build(&format!("{name} -> {target}")).unwrap();
        let result = deploy(Cache::default(), DeployOptions::default(), config);

        let expected = vec![];
        assert_eq!(result.mappings(), expected);
        assert!(!PathBuf::from(name).exists());
    }

    #[test]
    #[serial]
    fn link_nonempty_dir() {
        setup();
        let target = format!("{DOTFILE_DIR}/nvim");
        fs::create_dir(format!("{DOTFILE_DIR}/nvim")).unwrap();

        let init_target = &format!("{DOTFILE_DIR}/nvim/init.lua");
        fs::write(init_target, "").unwrap();

        let name = format!("{HOME_DIR}/.config/nvim");

        let config = Config::build(&format!("{name} -> {target}")).unwrap();
        let result = deploy(Cache::default(), DeployOptions::default(), config);

        let init_link = &format!("{HOME_DIR}/.config/nvim/init.lua");

        let expected = vec![Mapping::new(init_link, init_target)];
        assert_eq!(result.mappings(), expected);
        assert!(PathBuf::from(&init_link).exists());
        assert!(PathBuf::from(&init_link).is_symlink());
        assert!(PathBuf::from(&init_link)
            .canonicalize()
            .is_ok_and(|p| p == PathBuf::from(&init_target).canonicalize().unwrap()));
    }

    #[test]
    #[serial]
    fn link_nonempty_nested_dirs() {
        setup();
        let target = format!("{DOTFILE_DIR}/nvim");
        fs::create_dir(format!("{DOTFILE_DIR}/nvim")).unwrap();

        let init_target = &format!("{DOTFILE_DIR}/nvim/init.lua");
        fs::write(init_target, "").unwrap();

        let nested = format!("{DOTFILE_DIR}/nvim/lua/guy");
        fs::create_dir_all(nested).unwrap();

        let nested_target = format!("{DOTFILE_DIR}/nvim/lua/guy/nested.lua");
        fs::write(&nested_target, "").unwrap();

        let name = format!("{HOME_DIR}/.config/nvim");

        let config = Config::build(&format!("{name} -> {target}")).unwrap();
        let result = deploy(Cache::default(), DeployOptions::default(), config);

        let init_link = format!("{HOME_DIR}/.config/nvim/init.lua");
        let nested_link = format!("{HOME_DIR}/.config/nvim/lua/guy/nested.lua");

        let init_mapping = Mapping::new(&init_link, init_target);
        assert!(result.contains(&init_mapping));
        assert!(PathBuf::from(&init_link).exists());
        assert!(PathBuf::from(&init_link).is_symlink());
        assert!(PathBuf::from(&init_link)
            .canonicalize()
            .is_ok_and(|p| p == PathBuf::from(&init_target).canonicalize().unwrap()));

        let nested_mapping = Mapping::new(&nested_link, &nested_target);
        assert!(result.contains(&nested_mapping));
        assert!(PathBuf::from(&nested_link).exists());
        assert!(PathBuf::from(&nested_link).is_symlink());
        assert!(PathBuf::from(&nested_link)
            .canonicalize()
            .is_ok_and(|p| p == PathBuf::from(&nested_target).canonicalize().unwrap()));
    }

    #[test]
    #[serial]
    fn multiple_mappings() {
        setup();

        let target = format!("{DOTFILE_DIR}/nvim");
        fs::create_dir(format!("{DOTFILE_DIR}/nvim")).unwrap();

        let init_target = &format!("{DOTFILE_DIR}/nvim/init.lua");
        fs::write(init_target, "").unwrap();

        let name = format!("{HOME_DIR}/.config/nvim");

        let target2 = format!("{DOTFILE_DIR}/.vimrc");
        fs::write(&target2, "").unwrap();

        let name2 = format!("{HOME_DIR}/.vimrc");

        let config = Config::build(&format!(
            "{name} -> {target}
            {name2} -> {target2}"
        ))
        .unwrap();
        let result = deploy(Cache::default(), DeployOptions::default(), config);

        let init_link = format!("{name}/init.lua");

        let init_mapping = Mapping::new(&init_link, init_target);
        assert!(result.contains(&init_mapping));
        assert!(PathBuf::from(&init_link).exists());
        assert!(PathBuf::from(&init_link).is_symlink());
        assert!(PathBuf::from(&init_link)
            .canonicalize()
            .is_ok_and(|p| p == PathBuf::from(&init_target).canonicalize().unwrap()));

        let vimrc_mapping = Mapping::new(&name2, &target2);
        assert!(result.contains(&vimrc_mapping));
        assert!(PathBuf::from(&name2).exists());
        assert!(PathBuf::from(&name2).is_symlink());
        assert!(PathBuf::from(&name2)
            .canonicalize()
            .is_ok_and(|p| p == PathBuf::from(&target2).canonicalize().unwrap()));
    }
}
