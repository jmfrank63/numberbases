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
use num_traits::ToPrimitive;
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
        default_value = "10",
        conflicts_with = "config_file"
    )]
    source_base: isize,

    /// Target base
    #[clap(
        short = 't',
        long = "target_base",
        default_value = "10",
        conflicts_with = "config_file"
    )]
    target_base: isize,

    /// number to convert, this is mandatory
    source_number: String,

    /// Source alphabet, mandatory if base is greater than 10
    #[clap(short = 'a', long = "source_alphabet", default_value = HEX_ALPHABET, conflicts_with = "config_file")]
    source_alphabet: String,

    /// Alphabet, read from a File
    #[clap(short = 'b', long = "target_alphabet", default_value = HEX_ALPHABET, conflicts_with = "config_file")]
    target_alphabet: String,

    /// Alphabet, read from a File
    #[clap(short = 'c', long = "config_file", conflicts_with_all = &["source_base", "target_base", "source_alphabet", "target_alphabet"], default_value = "config.json")]
    config_file: String,
}

#[derive(Deserialize)]
struct Alphabet {
    value: i32,
    representation: String,
}

#[derive(Deserialize)]
struct System {
    base: String,
    symbols: Vec<Alphabet>,
}

#[derive(Deserialize)]
struct Config {
    source: System,
    target: System,
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

pub trait FromBase {
    fn from_base(number: &BigInt, base: &BigInt) -> Result<Self>
    where
        Self: Sized;
}

pub trait ToBase {
    fn to_base(&self, base: &BigInt) -> Result<BigInt>;
}

impl FromBase for BigInt {
    fn from_base(number: &BigInt, base: &BigInt) -> Result<Self> {
        let mut result = BigInt::zero();
        let mut multiplier = BigInt::one();
        let mut number = number.clone();

        while !number.is_zero() {
            let digit = (&number % base).to_u64().ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Digit too large",
            ))?;
            result = result + digit * &multiplier;
            multiplier = multiplier * base;
            number = number / base;
        }

        Ok(result)
    }
}

impl ToBase for BigInt {
    fn to_base(&self, base: &BigInt) -> Result<BigInt> {
        let mut result = BigInt::zero();
        let mut multiplier = BigInt::one();
        let mut number = self.clone();

        while !number.is_zero() {
            let digit = (&number % base).to_u64().ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Digit too large",
            ))?;
            result = result + digit * &multiplier;
            multiplier = multiplier * base;
            number = number / base;
        }

        Ok(result)
    }
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    println!("{:?}", opt);

    let source_alphabet_vector = if opt.source_alphabet.contains(',') {
        opt.source_alphabet.split(',').map(String::from).collect::<Vec<String>>()
    } else {
        opt.source_alphabet.chars().map(String::from).collect::<Vec<String>>()
    };

    let mut source_alphabet_hash: HashMap<String, u64> = HashMap::new();
    for symbol in source_alphabet_vector.clone().into_iter().enumerate() {
        source_alphabet_hash.insert(symbol.1, symbol.0 as u64);
    }

    let target_alphabet_vector = if opt.target_alphabet.contains(',') {
        opt.target_alphabet.split(',').map(String::from).collect::<Vec<String>>()
    } else {
        opt.target_alphabet.chars().map(String::from).collect::<Vec<String>>()
    };

    let mut target_alphabet_hash: HashMap<String, isize> = HashMap::new();
    for symbol in target_alphabet_vector.clone().into_iter().enumerate() {
        target_alphabet_hash.insert(symbol.1, symbol.0 as isize);
    }

    let source_base_sign = match opt.source_base {
        base if base < 0 => num_bigint::Sign::Minus,
        base if base > 0 => num_bigint::Sign::Plus,
        _ => return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Source base cannot be zero",
        )),
    };

    let target_base_sign = match opt.target_base {
        base if base < 0 => num_bigint::Sign::Minus,
        base if base > 0 => num_bigint::Sign::Plus,
        _ => return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Target base cannot be zero",
        )),
    };

    let source_base = BigInt::from(opt.source_base);
    let target_base = BigInt::from(opt.target_base);

    let source_number = if opt.source_number.contains(",") {
        opt.source_number.split(',').map(String::from).collect::<Vec<String>>()
    } else {
        opt.source_number.chars().map(String::from).collect::<Vec<String>>()
    };

    let source_number_sign = match source_number[0].as_str() {
        "-" => num_bigint::Sign::Minus,
        _ => num_bigint::Sign::Plus,
    };

    let source_number = if source_number[0].as_str() == "-" {
        source_number[1..].to_vec()
    } else {
        source_number
    };

    let convert_number = source_number.iter().map(|x| source_alphabet_hash[x]).collect::<Vec<u64>>();


    let mut convert = Convert::new(opt.source_base.abs() as u64, opt.target_base.abs() as u64);
    let output = convert.convert::<u64,u64>(&convert_number);
    let output_string = output.iter().map(|x| target_alphabet_vector[*x as usize].clone()).collect::<Vec<String>>().join("");
    let output_string = if source_number_sign == num_bigint::Sign::Minus {
        format!("-{}", output_string)
    } else {
        output_string
    };

    let lua = Lua::new();
    if let Err(e) = lua.load(LUA_BIGNUM).exec() {
        panic!("Error loading lua library: {}", e)
    }

    let config_file_path = opt.config_file;
    // Read the configuration file
    let config_file = fs::read_to_string(config_file_path)?;
    let config: Config = serde_json::from_str(&config_file)?;

    let max_position = 101;
    for position in 0..max_position {
        let base = calculate_base(&lua, &config.source.base, position)?;
        println!("Base for position {} is {}", position, base);
    }

    for position in 0..max_position {
        let base = calculate_base(&lua, &config.target.base, position)?;
        println!("Base for position {} is {}", position, base);
    }

    // TODO: Use the source and target systems to convert positionbers

    println!("{}\n{}", config.source.base, config.target.base);
    println!("Output: {}", output_string);
    println!("Target Hash: {:?}", target_alphabet_hash);
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
