use std::{path::PathBuf, io, fs, os::unix::fs::symlink, };

use config::Config;

pub mod config;

#[derive(Debug, PartialEq)]
pub enum DeployResult<'a> {
    All,
    Some { created: Vec<(&'a PathBuf, &'a PathBuf)> },
    None
}

pub fn deploy(config: &Config) -> DeployResult<'_> {
    let mut created = Vec::new();

    let valid_mappings = config.mappings().iter()
        // Source exists
        .filter_map(|(s, d) | { if s.exists() {
            Some((s, d))
        } else {
            // TODO: Print?
            None
        } } )
        // Destination doesn't exist
        .filter_map(|(s, d) | { if !d.exists() {
            Some((s, d))
        } else {
            // TODO: Print?
            None
        } } );

    for (src, dst) in valid_mappings {
        if let Err(e) = symlink(src, dst) {
            // TODO: Print
            println!("{e}");
        } else {
            created.push((src, dst));
        }
    }

    if created.len() == config.mappings().len() {
        DeployResult::All
    } else if !created.is_empty() {
        DeployResult::Some { created }
    } else {
        DeployResult::None
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::{fs, path::Path};

    use super::*;

    const TEST_DIR: &str = "test_dir";

    fn setup() {
        if Path::new(TEST_DIR).exists() {
            fs::remove_dir_all(TEST_DIR).unwrap();
        }
        fs::create_dir(TEST_DIR).unwrap();
    }

    #[test]
    #[serial]
    fn dont_link_if_exists() {
        setup();
        let src = PathBuf::from(TEST_DIR.to_string() + "/src");
        let dst = PathBuf::from(TEST_DIR.to_string() + "/dst");
        fs::write(src, "").unwrap();
        fs::write(dst, "").unwrap();

        let config = Config::build(&format!("{TEST_DIR}/src -> {TEST_DIR}/dst")).unwrap();
        let result = deploy(&config);
        let expected = DeployResult::None;
        assert_eq!(result, expected);
    }
}
