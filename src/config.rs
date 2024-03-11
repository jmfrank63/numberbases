use crate::consts::{DECIMAL_ALPHABET, DEFAULT_BASE};
use serde::Deserialize;
use std::{
    fmt::{self, Display, Formatter},
    fs,
};

use crate::CommandLineArgs;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct System {
    kind: String,
    base: Option<isize>,
    function: Option<String>,
    alphabet: Vec<(isize, char)>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Config {
    source: System,
    target: System,
}

impl Config {
    pub fn new(command_line_args: CommandLineArgs) -> Result<Self, &'static str> {
        if command_line_args.config_file.is_some() {
            let config_file_path = command_line_args
                .config_file
                .expect("Config file path not provided");
            let config_file =
                fs::read_to_string(config_file_path).expect("Failed to read config file");
            let config: Config =
                serde_json::from_str(&config_file).expect("Failed to parse config file");
            Ok(config)
        } else {
            let source_base = match command_line_args.source_base {
                Some(v) => v,
                None => match &command_line_args.source_alphabet {
                    Some(alphabet) => alphabet.len() as isize,
                    None => DEFAULT_BASE,
                },
            };

            if source_base < 0 && command_line_args.source_number.starts_with('-') {
                return Err("Both base and number cannot be negative");
            }

            let target_base = match command_line_args.target_base {
                Some(v) => v,
                None => match &command_line_args.target_alphabet {
                    Some(alphabet) => alphabet.len() as isize,
                    None => DEFAULT_BASE,
                },
            };
            let source_alphabet = match &command_line_args.source_alphabet {
                Some(alphabet) => alphabet
                    .chars()
                    .enumerate()
                    .map(|x| (x.0 as isize, x.1))
                    .collect(),
                None => DECIMAL_ALPHABET
                    .chars()
                    .enumerate()
                    .take(source_base as usize)
                    .map(|x| (x.0 as isize, x.1))
                    .collect(),
            };

            let target_alphabet = match &command_line_args.target_alphabet {
                Some(alphabet) => alphabet
                    .chars()
                    .enumerate()
                    .map(|x| (x.0 as isize, x.1))
                    .collect(),
                None => DECIMAL_ALPHABET
                    .chars()
                    .enumerate()
                    .take(target_base as usize)
                    .map(|x| (x.0 as isize, x.1))
                    .collect(),
            };
            Ok(Self {
                source: System {
                    kind: "constant".to_string(),
                    base: Some(source_base),
                    function: None,
                    alphabet: source_alphabet,
                },
                target: System {
                    kind: "constant".to_string(),
                    base: Some(target_base),
                    function: None,
                    alphabet: target_alphabet,
                },
            })
        }
    }

    pub fn get_source(&self) -> &System {
        &self.source
    }

    pub fn get_target(&self) -> &System {
        &self.target
    }

    pub fn get_source_base(&self) -> isize {
        self.source.base.unwrap_or(DEFAULT_BASE)
    }

    pub fn get_target_base(&self) -> isize {
        self.target.base.unwrap_or(DEFAULT_BASE)
    }

    pub fn get_source_alphabet(&self) -> &Vec<(isize, char)> {
        &self.source.alphabet
    }

    pub fn get_target_alphabet(&self) -> &Vec<(isize, char)> {
        &self.target.alphabet
    }
}

impl System {
    pub fn get_base(&self) -> isize {
        self.base.unwrap_or(DEFAULT_BASE)
    }

    pub fn get_alphabet(&self) -> &Vec<(isize, char)> {
        &self.alphabet
    }

    pub fn get_alphabet_letter_by_value(&self, value: isize) -> char {
        self.alphabet
            .iter()
            .find(|x| x.0 == value)
            .expect("Value not found in alphabet")
            .1
            .clone()
    }

    pub fn get_value_by_alphabet_letter(&self, letter: char) -> isize {
        self.alphabet
            .iter()
            .find(|x| x.1 == letter)
            .expect("Letter not found in alphabet")
            .0
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Source: {:?}\nTarget: {:?}", self.source, self.target)
    }
}

impl Display for System {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Kind: {}\nBase: {:?}\nFunction: {:?}\nAlphabet: {:?}",
            self.kind, self.base, self.function, self.alphabet
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let command_line_args = CommandLineArgs {
            source_base: Some(2),
            target_base: Some(10),
            source_number: "1010".to_string(),
            source_alphabet: Some("01".to_string()),
            target_alphabet: Some("0123456789".to_string()),
            config_file: None,
        };

        let config = Config::new(command_line_args);

        assert_eq!(config.clone().unwrap().source.base.unwrap(), 2);
        assert_eq!(config.clone().unwrap().target.base.unwrap(), 10);
        assert_eq!(config.clone().unwrap().source.alphabet.len(), 2);
        assert_eq!(config.clone().unwrap().target.alphabet.len(), 10);
    }

    #[test]
    fn test_base_inferred_from_alphabet() {
        let command_line_args = CommandLineArgs {
            source_base: None,
            target_base: None,
            source_number: "1010".to_string(),
            source_alphabet: Some("01".to_string()),
            target_alphabet: Some("01234567".to_string()),
            config_file: None,
        };

        let config = Config::new(command_line_args);

        // The base should be inferred from the length of the alphabet
        assert_eq!(config.clone().unwrap().source.base.unwrap(), 2);
        assert_eq!(config.clone().unwrap().target.base.unwrap(), 8);
    }

    #[test]
    fn test_base_defaults_to_10() {
        let command_line_args = CommandLineArgs {
            source_base: None,
            target_base: None,
            source_number: "1010".to_string(),
            source_alphabet: None,
            target_alphabet: None,
            config_file: None,
        };

        let config = Config::new(command_line_args);

        // The base should default to 10 if not provided
        assert_eq!(config.clone().unwrap().source.base.unwrap(), 10);
        assert_eq!(config.clone().unwrap().target.base.unwrap(), 10);
    }

    #[test]
    fn test_negative_base() {
        let command_line_args = CommandLineArgs {
            source_base: Some(-2),
            target_base: Some(-10),
            source_number: "1101".to_string(),
            source_alphabet: None,
            target_alphabet: None,
            config_file: None,
        };

        let config = Config::new(command_line_args);

        // The base should be -2 and -10 as provided
        assert_eq!(config.clone().unwrap().source.base.unwrap(), -2);
        assert_eq!(config.clone().unwrap().target.base.unwrap(), -10);
    }

    #[test]
    fn test_error_when_base_and_number_are_negative() {
        let command_line_args = CommandLineArgs {
            source_base: Some(-2),
            target_base: Some(10),
            source_number: "-1101".to_string(),
            source_alphabet: None,
            target_alphabet: None,
            config_file: None,
        };

        let config = Config::new(command_line_args);

        // An error should be returned when both base and number are negative
        assert!(config.is_err());
        assert_eq!(
            config.unwrap_err(),
            "Both base and number cannot be negative"
        );
    }
}
