use clap::Parser;

#[derive(Parser, Debug)]
struct Opt {
    /// Source base
    #[clap(short = 's', long = "source", default_value = "10")]
    source: u32,

    /// Target base
    #[clap(short = 't', long = "target", default_value = "10")]
    target: u32,

    /// Number to convert, this is mandatory
    number: u32,

    /// Alphabet, mandatory if base is greater than 10
    #[clap(short = 'a', long = "alphabet", conflicts_with_all = &["file", "generate"])]
    alphabet: Option<String>,

    /// Alphabet, read from a File
    #[clap(short = 'f', long = "file", conflicts_with_all = &["alphabet", "generate"])]
    file: Option<String>,

    /// Generate an alphabet
    #[clap(short = 'g', long = "generate", conflicts_with_all = &["alphabet", "file"])]
    generate: bool,
}

fn main() {
    let opt = Opt::parse();
    println!("{:?}", opt);
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
