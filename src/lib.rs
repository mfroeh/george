use std::{fs, io, os::unix::fs::symlink, path::PathBuf};

use config::Config;
use path_absolutize::*;
use walkdir::WalkDir;

pub mod cache;
pub mod config;

#[derive(Debug, PartialEq)]
pub struct Mapping {
    /// The name of the link (i.e. the destination)
    name: PathBuf,
    /// The target that will be pointed to
    target: PathBuf,
}

impl Mapping {
    pub fn new(name: &str, target: &str) -> Mapping {
        let name = shellexpand::tilde(name).to_string();
        let target = shellexpand::tilde(target).to_string();

        let name = PathBuf::from(name).absolutize().unwrap().into();
        let target = PathBuf::from(target).absolutize().unwrap().into();
        Mapping { name, target }
    }

    pub fn name(&self) -> &PathBuf {
        &self.name
    }

    pub fn destination(&self) -> &PathBuf {
        &self.name
    }

    pub fn target(&self) -> &PathBuf {
        &self.target
    }
}

#[derive(Debug, PartialEq)]
pub enum DeployResult {
    Some { created: Vec<PathBuf> },
    None,
}

pub fn deploy(config: &Config) -> DeployResult {
    let valid_mappings = config
        .mappings()
        .iter()
        .filter_map(|m| {
            if m.target().exists() {
                Some(m)
            } else {
                // TODO: Print?
                None
            }
        })
        // Destination doesn't exist
        .filter_map(|m| {
            if !m.name().exists() {
                Some(m)
            } else {
                // TODO: Print?
                None
            }
        });

    let mut created = Vec::new();
    for mapping in valid_mappings {
        if let Ok(mut c) = create_links(mapping) {
            created.append(&mut c);
        }
    }

    if created.len() > 0 {
        DeployResult::Some { created }
    } else {
        DeployResult::None
    }
}

fn create_links(mapping: &Mapping) -> io::Result<Vec<PathBuf>> {
    let mut created = Vec::new();

    if !mapping.target().is_dir() {
        if let Some(parent) = mapping.name().parent() {
            fs::create_dir_all(parent)?;
        }
        symlink(mapping.target(), mapping.name())?;
        created.push(mapping.name().into());
        Ok(created)
    } else {
        let target_base = mapping.target().to_str().unwrap();
        let link_base = mapping.name().to_str().unwrap();
        for target in WalkDir::new(mapping.target())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let name = target
                .path()
                .to_str()
                .unwrap()
                .replace(target_base, link_base);
            let name = PathBuf::from(name);

            if !name.parent().unwrap().exists() {
                fs::create_dir_all(name.parent().unwrap())?;
            }
            symlink(target.path(), &name)?;
            created.push(name);
        }
        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::{fs, path::Path};

    use super::*;

    const DOTFILE_DIR: &str = "/home/mo/test_dotfiles";
    const HOME_DIR: &str = "/home/mo/test_home";

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
    fn link_existing_file() {
        setup();
        let target = PathBuf::from(DOTFILE_DIR.to_string() + "/.zshrc");
        fs::create_dir(target).unwrap();

        let name = PathBuf::from(HOME_DIR.to_string() + "/.zshrc");
        fs::create_dir(name).unwrap();

        let config = Config::build(&format!("{HOME_DIR}/.zshrc -> {DOTFILE_DIR}/.zshrc")).unwrap();
        let result = deploy(&config);
        let expected = DeployResult::None;
        assert_eq!(result, expected);
    }

    #[test]
    #[serial]
    fn link_single_file() {
        setup();
        let target = PathBuf::from(DOTFILE_DIR.to_string() + "/.vimrc");
        fs::write(target, "hihi").unwrap();

        let name = PathBuf::from(format!("{HOME_DIR}/.vimrc"));

        let config = Config::build(&format!("{HOME_DIR}/.vimrc -> {DOTFILE_DIR}/.vimrc")).unwrap();
        let result = deploy(&config);
        let expected = DeployResult::Some {
            created: vec![name],
        };
        assert_eq!(result, expected);
    }

    #[test]
    #[serial]
    fn link_file_in_nested_dir() {
        setup();
        let target = PathBuf::from(DOTFILE_DIR.to_string() + "/my-config/neovim");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("init.lua"), "hihi").unwrap();

        let name = PathBuf::from(HOME_DIR.to_string() + "/.config/nvim/init.lua");
        let config = Config::build(&format!(
            "{HOME_DIR}/.config/nvim/init.lua -> {DOTFILE_DIR}/my-config/neovim/init.lua"
        ))
        .unwrap();
        let result = deploy(&config);
        let expected = DeployResult::Some {
            created: vec![name],
        };
        assert_eq!(result, expected);
    }

    #[test]
    #[serial]
    fn link_empty_dir() {
        setup();
        let target = PathBuf::from(DOTFILE_DIR.to_string() + "/my-config");
        fs::create_dir_all(target).unwrap();

        let config =
            Config::build(&format!("{HOME_DIR}/.config -> {DOTFILE_DIR}/my-config")).unwrap();
        let result = deploy(&config);
        let expected = DeployResult::None;
        assert_eq!(result, expected);
    }

    #[test]
    #[serial]
    fn link_nonempty_dir() {
        setup();
        let target = PathBuf::from(DOTFILE_DIR.to_string() + "/my-config");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("somefile"), "hihi").unwrap();

        let name = PathBuf::from(HOME_DIR.to_string() + "/.config/myfile");
        let config = Config::build(&format!(
            "{HOME_DIR}/.config/myfile -> {DOTFILE_DIR}/my-config/somefile"
        ))
        .unwrap();

        let result = deploy(&config);
        let expected = DeployResult::Some {
            created: vec![name],
        };
        assert_eq!(result, expected);
    }

    #[test]
    #[serial]
    fn link_nonempty_nested_dirs() {
        setup();
        let target = PathBuf::from(DOTFILE_DIR.to_string() + "/my-config/nvim");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("init.lua"), "hihi").unwrap();

        let name = PathBuf::from(HOME_DIR.to_string() + "/.config/nvim/init.lua");
        let config =
            Config::build(&format!("{HOME_DIR}/.config -> {DOTFILE_DIR}/my-config")).unwrap();
        let result = deploy(&config);
        let expected = DeployResult::Some {
            created: vec![name],
        };
        assert_eq!(result, expected);
    }

    #[test]
    #[serial]
    fn multiple_nested_dirs() {
        setup();
        let target = PathBuf::from(DOTFILE_DIR.to_string() + "/my-config/nvim");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("init.lua"), "hihi").unwrap();
        let target2 = target.join("dep");
        fs::create_dir_all(&target2).unwrap();
        fs::write(target2.join("testfile"), "").unwrap();

        let target3 = PathBuf::from(DOTFILE_DIR.to_string() + "/other/karabiner");
        fs::create_dir_all(&target3).unwrap();
        fs::write(target3.join("some.json"), "").unwrap();

        let name = PathBuf::from(HOME_DIR.to_string() + "/.config/nvim/init.lua");
        let name2 = PathBuf::from(HOME_DIR.to_string() + "/.config/nvim/dep/testfile");
        let name3 = PathBuf::from(HOME_DIR.to_string() + "/.config/karabiner/config.json");
        let config = Config::build(&format!(
            "{HOME_DIR}/.config -> {DOTFILE_DIR}/my-config
            {HOME_DIR}/.config/nvim -> {DOTFILE_DIR}/my-config/nvim
            {HOME_DIR}/.config/karabiner/config.json -> {DOTFILE_DIR}/other/karabiner/some.json"
        ))
        .unwrap();

        match deploy(&config) {
            DeployResult::Some { created } => {
                for name in [name, name2, name3] {
                    assert!(created.contains(&name))
                }
            }
            DeployResult::None => panic!(),
        }
    }
}
