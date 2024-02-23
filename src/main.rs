use clap::Parser;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

const ALPHABET: [&'static str; 175] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "А", "Б", "В", "Г", "Д", "Е", "Ж", "З", "И", "Й", "К", "Л", "М", "Н", "О", "П", "Р", "С", "Т", "У", "Ф", "Х", "Ц", "Ч", "Ш", "Щ", "Ъ", "Ы", "Ь", "Э", "Ю", "Я", "а", "б", "в", "г", "д", "е", "ж", "з", "и", "й", "к", "л", "м", "н", "о", "п", "р", "с", "т", "у", "ф", "х", "ц", "ч", "ш", "щ", "ъ", "ы", "ь", "э", "ю", "я", "Α", "Β", "Γ", "Δ", "Ε", "Ζ", "Η", "Θ", "Ι", "Κ", "Λ", "Μ", "Ν", "Ξ", "Ο", "Π", "Ρ", "Σ", "Τ", "Υ", "Φ", "Χ", "Ψ", "Ω", "α", "β", "γ", "δ", "ε", "ζ", "η", "θ", "ι", "κ", "λ", "μ", "ν", "ξ", "ο", "π", "ρ", "ς", "σ", "τ", "υ", "φ", "χ", "ψ", "ω"];
#[derive(Parser, Debug)]
struct Opt {
    /// Source base
    #[clap(short = 's', long = "source_base", default_value = "10")]
    source: usize,

    /// Source negative
    #[clap(short = 'n', long = "source_negative")]
    source_negative: bool,

    /// Target base
    #[clap(short = 't', long = "target_base", default_value = "10")]
    target: usize,

    /// Target negative
    #[clap(short = 'm', long = "target_negative")]
    target_negative: bool,

    /// Number to convert, this is mandatory
    number: usize,

    /// Number is negative
    #[clap(short = 'i', long = "number_negative")]
    number_negative: bool,

    /// Source alphabet, mandatory if base is greater than 10
    #[clap(short = 'a', long = "source_alphabet", conflicts_with = "source_file")]
    source_alphabet: Option<String>,

    /// target alphabet, mandatory if base is greater than 10
    #[clap(short = 'b', long = "target_alphabet", conflicts_with = "target_file")]
    target_alphabet: Option<String>,

    /// Alphabet, read from a File
    #[clap(short = 'f', long = "source_file", conflicts_with = "source_alphabet")]
    source_file: Option<String>,

    /// Alphabet, read from a File
    #[clap(short = 'g', long = "target_file", conflicts_with = "target_alphabet")]
    target_file: Option<String>,
}

use std::io;

fn read_alphabet(
    file: Option<String>,
    alphabet: Option<String>,
    length: usize,
) -> Result<Vec<String>> {
    let alphabet = match file {
        Some(ref file) => {
            let path = Path::new(file);
            let alphabet_file = std::fs::read_to_string(&path)?;
            let alphabet: Vec<String> = alphabet_file
                .lines()
                .map(|line| line.trim().into())
                .collect();
            if length < alphabet.len() {
                alphabet.split_at(length).0.to_vec() // Truncate to length
            } else {
                alphabet
            }
        }
        None => match alphabet {
            Some(alphabet) => {
                let alphabet: Vec<String> = alphabet.split(',').map(|c| c.to_string()).collect();
                if length < alphabet.len() {
                    alphabet.split_at(length).0.to_vec() // Truncate to length
                } else {
                    alphabet
                }
            }
            None => {
                // Generate a sequential ASCII alphabet
                ALPHABET[..length].iter().map(|c| c.to_string()).collect()
            }
        },
    };

    alphabet_is_unique(&alphabet)?;
    alphabet_has_length(&alphabet, length)?;

    Ok(alphabet)
}

fn alphabet_is_unique(alphabet: &Vec<String>) -> std::io::Result<()> {
    for i in 0..alphabet.len() {
        for j in i + 1..alphabet.len() {
            if alphabet[i] == alphabet[j] {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Non unique symbol {}. Alphabet must be unique", alphabet[i]),
                ));
            }
        }
    }
    Ok(())
}

fn alphabet_has_length(alphabet: &Vec<String>, base: usize) -> std::io::Result<()> {
    if alphabet.len() != base as usize {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Alphabet has length {} but base is {}. Alphabet must have the same number of unique symbols",
                alphabet.len(),
                base
            ),
        ));
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let opt = Opt::parse();
    println!("{:?}", opt);


    let source_alphabet = read_alphabet(opt.source_file, opt.source_alphabet, opt.source)?;
    let target_alphabet = read_alphabet(opt.target_file, opt.target_alphabet, opt.target)?;


    if opt.source_negative {
        for ch in &source_alphabet {
            print!("-{} ", ch);
        }
    } else {
        for ch in &source_alphabet {
            print!("{} ", ch);
        }
    }
    println!();

    if opt.target_negative {
        for ch in &target_alphabet {
            print!("-{} ", ch);
        }
    } else {
        for ch in &target_alphabet {
            print!("{} ", ch);
        }
    }
    println!();

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

    #[test]
    fn test_alphabet_is_unique() {
        let alphabet1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert!(alphabet_is_unique(&alphabet1).is_ok());

        let alphabet2 = vec!["a".to_string(), "b".to_string(), "a".to_string()];
        assert!(alphabet_is_unique(&alphabet2).is_err());

        let alphabet3 = vec![];
        assert!(alphabet_is_unique(&alphabet3).is_ok());
    }

    #[test]
    fn test_alphabet_has_length() {
        let alphabet1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert!(alphabet_has_length(&alphabet1, 3).is_ok());

        let alphabet2 = vec!["a".to_string(), "b".to_string()];
        assert!(alphabet_has_length(&alphabet2, 3).is_err());
    }
}
