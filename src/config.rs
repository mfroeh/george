use std::{
    error::Error,
    fmt::Display,
};
use crate::Mapping;

#[derive(Debug, PartialEq)]
pub struct Config {
    mappings: Vec<Mapping>,
}

impl Config {
    pub fn mappings(&self) -> &Vec<Mapping> {
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
        let mut mappings = Vec::new();
        for (i, line) in content.lines().filter(|l| !l.is_empty()).enumerate() {
            let mapping: Vec<&str> = line.split("->").collect();
            if mapping.len() != 2 {
                return Err(ConfigFormatError::new(line, i + 1));
            }

            let name = mapping[0].trim();
            let target = mapping[1].trim();
            mappings.push(Mapping::new(name, target));
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
name -> target
~/.config -> my-config

from/here/ -> to/there
";
        let result = Config::build(config);

        let mappings = vec![
            Mapping::new("name", "target"),
            Mapping::new("~/.config", "my-config"),
            Mapping::new("from/here/", "to/there"),
        ];

        let expected = Ok(Config { mappings });
        assert_eq!(result, expected);
    }
}
