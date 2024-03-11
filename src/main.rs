#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_mut)]

mod bignum;

use bignum::LUA_BIGNUM;

use clap::Parser;
use convert_base::Convert;
use mlua::prelude::*;
use num_bigint::BigInt;
use num_traits::identities::{One, Zero};
use num_traits::{Signed, ToPrimitive};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::Result;
use std::io::{Error, ErrorKind};

const DECIMAL_ALPHABET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzАБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдежзийклмнопрстуфхцчшщъыьэюя";

#[derive(Parser, Debug)]
struct Opt {
    /// Source base
    #[clap(short = 's', long = "source_base", conflicts_with_all = &["config_file", "source_alphabet"])]
    source_base: Option<isize>,

    /// Target base
    #[clap(short = 't', long = "target_base", conflicts_with_all = &["config_file", "target_alphabet"])]
    target_base: Option<isize>,

    /// number to convert, this is mandatory
    source_number: String,

    /// Source alphabet as comma separated list
    #[clap(short = 'a', long = "source_alphabet", conflicts_with_all = &["config_file", "source_base"])]
    source_alphabet: Option<String>,

    /// Target alphabet as comma separated list
    #[clap(short = 'b', long = "target_alphabet", conflicts_with_all = &["config_file", "target_base"])]
    target_alphabet: Option<String>,

    /// Supply all arguments via a config file
    #[clap(short = 'c', long = "config_file", conflicts_with_all = &["source_base", "target_base", "source_alphabet", "target_alphabet"])]
    config_file: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Alphabet {
    value: i32,
    symbol: String,
}

#[derive(Deserialize, Debug)]
struct System {
    kind: String,
    base: Option<isize>,
    function: Option<String>,
    alphabet: Vec<Alphabet>,
}

#[derive(Deserialize, Debug)]
struct Config {
    source: System,
    target: System,
}

impl Config {
    pub fn new(opt: Opt) -> Self {
        if opt.config_file.is_some() {
            let config_file = fs::read_to_string(opt.config_file.unwrap()).unwrap();
            let mut config: Config = serde_json::from_str(&config_file).unwrap();
            config
        } else {
            let source_base = match opt.source_base {
                Some(v) => v,
                None => {
                    match &opt.source_alphabet {
                        Some(alphabet) => {
                            alphabet.len() as isize
                        }
                        None => 10,
                    }
                },
            };

            
            let target_base = match opt.target_base {
                Some(v) => v,
                None => {
                    match &opt.target_alphabet {
                        Some(alphabet) => {
                            alphabet.len() as isize
                        }
                        None => 10,
                    }
                },
            };
            let source_alphabet = match &opt.source_alphabet {
                Some(alphabet) => {
                    alphabet
                        .chars()
                        .enumerate()
                        
                        .map(|x| Alphabet {
                            value: x.0 as i32,
                            symbol: x.1.to_string(),
                        })
                        .collect()
                }
                None => DECIMAL_ALPHABET
                    .chars()
                    .enumerate()
                    .take(source_base as usize)
                    .map(|x| Alphabet {
                        value: x.0 as i32,
                        symbol: x.1.to_string(),
                    })
                    .collect(),
            };

            let target_alphabet = match &opt.target_alphabet {
                Some(alphabet) => {
                    alphabet
                        .chars()
                        .enumerate()
                        .map(|x| Alphabet {
                            value: x.0 as i32,
                            symbol: x.1.to_string(),
                        })
                        .collect()
                }
                None => DECIMAL_ALPHABET
                    .chars()
                    .enumerate()
                    .take(target_base as usize)
                    .map(|x| Alphabet {
                        value: x.0 as i32,
                        symbol: x.1.to_string(),
                    })
                    .collect(),
            };

            Self {
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
            }
        }
    }
}

#[derive(Debug)]
struct Converter<'a> {
    config: &'a Config,
    lua: Option<Lua>,
    number: BigInt,
}

impl<'a> Converter<'a> {
    pub fn new(config: &'a Config, number: BigInt) -> Self {

        Self { config, lua: None, number }
    }
}

fn calculate_base(lua: &Lua, function: &str, position: isize) -> Result<BigInt> {
    let calculate_base: mlua::Function = match lua.load(function).eval() {
        Ok(f) => f,
        Err(e) => panic!("Error loading lua function: {}", e),
    };
    let big_string: String = match calculate_base.call(position) {
        Ok(v) => v,
        Err(e) => panic!("Error calling lua function: {}", e),
    };
    let big_base = BigInt::parse_bytes(big_string.as_bytes(), 10)
        .ok_or(Error::new(ErrorKind::InvalidData, "Invalid value"))?;
    Ok(big_base)
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    println!("{:?}", opt);

    let config = Config::new(opt);
    println!("{:#?}", config);
   
    // TODO: Use the source and target systems to convert positionbers

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Opt::command().debug_assert()
    }
}
