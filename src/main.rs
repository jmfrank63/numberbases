
mod converter;
mod config;
mod consts;

use clap::Parser;
use std::io::Result;
use crate::config::Config;

#[derive(Parser, Debug)]
struct CommandLineArgs {
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


fn main() -> Result<()> {
    let command_line_args = CommandLineArgs::parse();
    println!("{:?}", command_line_args);

    let config = Config::new(command_line_args);
    println!("{:#?}", config);
    let config = &config.unwrap();
    let converter = converter::Converter::new(&config, 0.into());

    println!("{:#?}", converter);
   
    // TODO: Use the source and target systems to convert positions

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        CommandLineArgs::command().debug_assert()
    }
}
