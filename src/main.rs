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

const HEX_ALPHABET: &str = "0123456789ABCDEF";
const HEX_VALUES: &str = "0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15";

#[derive(Parser, Debug)]
struct Opt {
    /// Source base
    #[clap(
        short = 's',
        long = "source_base",
        conflicts_with = "config_file"
    )]
    source_base: Option<isize>,

    /// Target base
    #[clap(
        short = 't',
        long = "target_base",
        conflicts_with = "config_file"
    )]
    target_base: Option<isize>,

    /// number to convert, this is mandatory
    source_number: String,

    /// Source alphabet, mandatory if base is greater than 10
    #[clap(short = 'a', long = "source_alphabet", default_value = HEX_ALPHABET, conflicts_with = "config_file")]
    source_alphabet: String,

    /// Alphabet, read from a File
    #[clap(short = 'b', long = "target_alphabet", default_value = HEX_ALPHABET, conflicts_with = "config_file")]
    target_alphabet: String,

    /// Alphabet, read from a File
    #[clap(short = 'c', long = "config_file", conflicts_with_all = &["source_base", "target_base", "source_alphabet", "target_alphabet"])]
    config_file: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Alphabet {
    value: i32,
    representation: String,
}

#[derive(Deserialize, Debug)]
struct System {
    kind: String,
    base: Option<isize>,
    function: Option<String>,
    symbols: Vec<Alphabet>,
}

#[derive(Deserialize, Debug)]
struct Config {
    source: System,
    target: System,
}

impl Config {
    pub fn new(
        source_base: isize,
        target_base: isize,
        source_alphabet: Vec<String>,
        target_alphabet: Vec<String>,
    ) -> Self {
        let source_alphabet: Vec<Alphabet> = source_alphabet
            .into_iter()
            .enumerate()
            .map(|x| Alphabet {
                value: x.0 as i32,
                representation: x.1,
            })
            .collect();

        let target_alphabet: Vec<Alphabet> = target_alphabet
            .into_iter()
            .enumerate()
            .map(|x| Alphabet {
                value: x.0 as i32,
                representation: x.1,
            })
            .collect();

        Self {
            source: System {
                kind: "constant".to_string(),
                base: Some(source_base),
                function: None,
                symbols: source_alphabet,
            },
            target: System {
                kind: "constant".to_string(),
                base: Some(target_base),
                function: None,
                symbols: target_alphabet,
            },
        }
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

    let source_alphabet_vector = if opt.source_alphabet.contains(',') {
        opt.source_alphabet
            .split(',')
            .map(String::from)
            .collect::<Vec<String>>()
    } else {
        opt.source_alphabet
            .chars()
            .map(String::from)
            .collect::<Vec<String>>()
    };

    let mut source_alphabet_hash: HashMap<String, u64> = HashMap::new();
    for symbol in source_alphabet_vector.clone().into_iter().enumerate() {
        source_alphabet_hash.insert(symbol.1, symbol.0 as u64);
    }

    let target_alphabet_vector = if opt.target_alphabet.contains(',') {
        opt.target_alphabet
            .split(',')
            .map(String::from)
            .collect::<Vec<String>>()
    } else {
        opt.target_alphabet
            .chars()
            .map(String::from)
            .collect::<Vec<String>>()
    };

    let mut target_alphabet_hash: HashMap<String, isize> = HashMap::new();
    for symbol in target_alphabet_vector.clone().into_iter().enumerate() {
        target_alphabet_hash.insert(symbol.1, symbol.0 as isize);
    }

    let source_base_sign = match opt.source_base.unwrap_or(10) {
        base if base < 0 => num_bigint::Sign::Minus,
        base if base > 0 => num_bigint::Sign::Plus,
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Source base cannot be zero",
            ))
        }
    };

    let target_base_sign = match opt.target_base.unwrap_or(10) {
        base if base < 0 => num_bigint::Sign::Minus,
        base if base > 0 => num_bigint::Sign::Plus,
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Target base cannot be zero",
            ))
        }
    };

    let source_base = BigInt::from(opt.source_base.unwrap_or(10));
    let target_base = BigInt::from(opt.target_base.unwrap_or(10));

    let source_number = if opt.source_number.contains(",") {
        opt.source_number
            .split(',')
            .rev()
            .map(String::from)
            .collect::<Vec<String>>()
    } else {
        opt.source_number
            .chars()
            .rev()
            .map(String::from)
            .collect::<Vec<String>>()
    };
    println!("{:?}", source_number);

    let source_length = source_number.len() - 1;
    let source_number_sign = match source_number[source_length].as_str() {
        "-" => num_bigint::Sign::Minus,
        _ => num_bigint::Sign::Plus,
    };

    let source_number = if source_number[source_length].as_str() == "-" {
        source_number[..source_length].to_vec()
    } else {
        source_number
    };

    println!("{:?}", source_number);
    let convert_number = source_number
        .iter()
        .map(|x| source_alphabet_hash[x])
        .collect::<Vec<u64>>();

    let lua = Lua::new();
    if let Err(e) = lua.load(LUA_BIGNUM).exec() {
        panic!("Error loading lua library: {}", e)
    }

    let config = match opt.config_file {
        Some(path) => {
            let config_file = fs::read_to_string(path)?;
            serde_json::from_str(&config_file)?
        }
        None => Config::new(
            opt.source_base.unwrap(),
            opt.target_base.unwrap(),
            source_alphabet_vector.clone(),
            target_alphabet_vector.clone(),
        ),
    };

    println!("{:?}", config);
    let max_position = 101;
    let mut convert = match (config.source.kind.as_ref(), config.target.kind.as_ref()) {
        ("lua", "lua") => {
            let source_function = config.source.function.as_ref().ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No function provided",
            ))?;
            let target_function = config.target.function.as_ref().ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No function provided",
            ))?;
            for position in 0..max_position {
                let source_base = calculate_base(&lua, source_function, position)?;
                println!("Base for position {} is {}", position, source_base);
                let target_base = calculate_base(&lua, target_function, position)?;
                println!("Base for position {} is {}", position, target_base);
            }
            // TODO: Adopt convert to handle BigInt
            Convert::new(source_base.to_u64().unwrap(), target_base.to_u64().unwrap())
        },
        ("lua", "constant") => {
            let source_function = config.source.function.as_ref().ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No function provided",
            ))?;
            let target_base = config.target.base.ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No base provided",
            ))?;
            for position in 0..max_position {
                let source_base = calculate_base(&lua, source_function, position)?;
                println!("Base for position {} is {}", position, source_base);
            }
            Convert::new(source_base.to_u64().unwrap(), target_base as u64)
        },
        ("constant", "lua") => {
            let source_base = config.source.base.ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No base provided",
            ))?;
            let target_function = config.target.function.as_ref().ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No function provided",
            ))?;
            for position in 0..max_position {
                let target_base = calculate_base(&lua, target_function, position)?;
                println!("Base for position {} is {}", position, target_base);
            }
            Convert::new(source_base as u64, target_base.to_u64().unwrap())
        },
        ("constant", "constant") => {
            let source_base = config.source.base.ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No base provided",
            ))?;
            let target_base = config.target.base.ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No base provided",
            ))?;
            println!("Source Base: {:?}\nTarget Base: {:?}", source_base, target_base);
            Convert::new(source_base as u64, target_base as u64)
        },
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid source and target systems",
            ))
        }
    };
    let mut convert = Convert::new(10, 2);
    let output = convert.convert::<u64, u64>(&convert_number);
    let output = output.into_iter().rev().collect::<Vec<u64>>();
    let output_string = output
        .iter()
        .map(|x| target_alphabet_vector[*x as usize].clone())
        .collect::<Vec<String>>()
        .join("");
    let output_string = if source_number_sign == num_bigint::Sign::Minus {
        format!("-{}", output_string)
    } else {
        output_string
    };

    // TODO: Use the source and target systems to convert positionbers
    println!("Output: {}", output_string);
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
