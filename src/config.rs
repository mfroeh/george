use serde::{ser, Deserialize};
use std::{
    collections::HashMap,
    default,
    error::Error,
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
};
use toml::Table;

#[derive(Debug, PartialEq)]
pub struct Config {
    mappings: HashMap<PathBuf, PathBuf>,
}

impl Config {
    pub fn mappings(&self) -> &HashMap<PathBuf, PathBuf> {
        &self.mappings
    }
}

#[derive(Debug, PartialEq)]
pub struct ConfigFormatError {
    line_nr: usize,
    line: String,
}

impl ConfigFormatError {
    fn new(line: &str, line_nr: usize) -> ConfigFormatError {
        ConfigFormatError {
            line: line.to_string(),
            line_nr,
        }
    }
}

impl Display for ConfigFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config format error on line {0}: {1}",
            self.line_nr, self.line
        )
    }
}

impl Error for ConfigFormatError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Config {
    pub fn build(content: &str) -> Result<Config, ConfigFormatError> {
        let mut mappings = HashMap::new();
        for (i, line) in content.lines().filter(|l| !l.is_empty()).enumerate() {
            let mapping: Vec<&str> = line.split("->").collect();
            if mapping.len() != 2 {
                return Err(ConfigFormatError::new(line, i + 1));
            }

            let src = PathBuf::from(mapping[0].trim());
            let dst = PathBuf::from(mapping[1].trim());
            mappings.insert(src, dst);
        }

        let config = Config { mappings };
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_format_err() {
        let config = "
foo -> bar
fox -- tox
";

        let result = Config::build(config);
        let expected = Err(ConfigFormatError {
            line: "fox -- tox".to_string(),
            line_nr: 2,
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn good_config() {
        let config = "
src -> dest
.config -> ~/.config

from/here/ -> to/there
";
        let result = Config::build(config);

        let mut mappings = HashMap::new();
        mappings.insert("src".into(), "dest".into());
        mappings.insert(".config".into(), "~/.config".into());
        mappings.insert("from/here".into(), "to/there".into());

        let expected = Ok(Config { mappings });
        assert_eq!(result, expected);
    }
}
