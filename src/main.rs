use clap::Parser;
use std::io::Result;
use std::env;
use std::fs;
use mlua::prelude::*;
use serde::Deserialize;
use serde_json::Value;

const HEX_ALPHABET: [&'static str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"];
#[derive(Parser, Debug)]
struct Opt {
    /// Source base
    #[clap(short = 's', long = "source_base", default_value = "10")]
    source: isize,

    /// Target base
    #[clap(short = 't', long = "target_base", default_value = "10")]
    target: isize,

    /// Number to convert, this is mandatory
    number: isize,

    /// Source alphabet, mandatory if base is greater than 10
    #[clap(short = 'a', long = "source_alphabet", conflicts_with = "config_file")]
    source_alphabet: Option<String>,

    /// Alphabet, read from a File
    #[clap(short = 'b', long = "target_alphabet", conflicts_with = "config_file")]
    target_alphabet: Option<String>,

    /// Alphabet, read from a File
    #[clap(short = 'c', long = "config_file", conflicts_with_all = &["source_alphabet", "target_alphabet"])]
    config_file: Option<String>,
}

#[derive(Deserialize)]
struct Alphabet {
    position: i32,
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

fn calculate_base(lua: &Lua, base: &str, num: i32) -> Result<i32> {
    let calculate_base: mlua::Function = match lua.load(base).eval(){
        Ok(f) => f,
        Err(e) => panic!("Error loading lua function: {}", e)
    };
    let v: i32 = match calculate_base.call(num) {
        Ok(v) => v,
        Err(e) => panic!("Error calling lua function: {}", e)
    
    };
    Ok(v)
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    println!("{:?}", opt);
    println!("{:?}", HEX_ALPHABET);

    let lua = Lua::new();

    // Read the name of the configuration file from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Please provide the name of the configuration file as a command line argument");
    }
    let config_file_name = &args[1];

    // Read the configuration file
    let config_file = fs::read_to_string(config_file_name).expect("Could not read config file");
    let config: Config = serde_json::from_str(&config_file).expect("Could not parse config file");

    // Calculate the base for each position in the source system
    for alphabet in &config.source.symbols {
        let base = calculate_base(&lua, &config.source.base, alphabet.position)?;
        println!("The base for position {} in the source system is {}", alphabet.position, base);
    }

    // Calculate the base for each position in the target system
    for alphabet in &config.target.symbols {
        let base = calculate_base(&lua, &config.target.base, alphabet.position)?;
        println!("The base for position {} in the target system is {}", alphabet.position, base);
    }

    // TODO: Use the source and target systems to convert numbers

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