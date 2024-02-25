use clap::Parser;
use mlua::prelude::*;
use num_bigint::BigInt;
use serde::Deserialize;
use std::fs;
use std::io::Result;
use std::io::{Error, ErrorKind};

const HEX_ALPHABET: [&'static str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F",
];
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

    /// positionber to convert, this is mandatory
    positionber: isize,

    /// Source alphabet, mandatory if base is greater than 10
    #[clap(short = 'a', long = "source_alphabet", conflicts_with = "config_file")]
    source_alphabet: Option<String>,

    /// Alphabet, read from a File
    #[clap(short = 'b', long = "target_alphabet", conflicts_with = "config_file")]
    target_alphabet: Option<String>,

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

fn calculate_base(lua: &Lua, base: &str, position: isize) -> Result<BigInt> {
    let calculate_base: mlua::Function = match lua.load(base).eval() {
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
    println!("{:?}", HEX_ALPHABET);

    let lua = Lua::new();
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

    println!("{} \n {}", config.source.base, config.target.base);
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
