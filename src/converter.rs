mod bignum;

use crate::config::Config;
use bignum::LUA_BIGNUM;

use mlua::prelude::*;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::io::{Error, ErrorKind};
use std::io::Result;
use num_integer::Integer;

#[derive(Debug)]
pub(crate) struct Converter<'a> {
    config: &'a Config,
    lua: Lua,
    number: BigInt,
}

impl<'a> Converter<'a> {
    pub fn new(config: &'a Config, number: BigInt) -> Self {
        let lua = Lua::new();
        lua.load(LUA_BIGNUM).exec().expect("Error loading lua bignum library");
        Self { config, lua, number }
    }

    fn calculate_base(self, function: &str, position: isize) -> Result<BigInt> {
        let calculate_base: mlua::Function = match self.lua.load(function).eval() {
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

    pub fn print_number_in_target_base(&self) -> String {
        let base = self.config.get_source_base();
        let mut number = self.number.clone();
        let mut result = Vec::new();

        while number != BigInt::from(0) {
            let (quotient, remainder) = number.div_rem(&BigInt::from(base));
            number = quotient;
            result.push(remainder.to_str_radix(10));
        }

        result.reverse();
        result.join("")
    }

    pub fn print_number_in_target_alphabet(&self) -> String {
        let alphabet = &self.config.get_target_alphabet();
        let base = BigInt::from(alphabet.len());
        let mut number = self.number.clone();
        let mut result = String::new();

        while number != BigInt::from(0) {
            let (quotient, remainder) = number.div_rem(&base);
            number = quotient;
            let digit = remainder.to_usize().unwrap();
            result.push(alphabet[digit].1.clone());
        }

        result.chars().rev().collect::<String>()
    }

}

#[cfg(test)]
mod tests {
    use crate::CommandLineArgs;

    use super::*;
    use num_bigint::BigInt;
    use std:: str::FromStr;

    #[test]
    fn test_calculate_base_1() {
        let lua_function = r#"f = function (n) n = BigNum.new(n) base = BigNum.new(10^6) return tostring(base^n) end return f"#;

        let command_line_args = CommandLineArgs {
            source_base: Some(2),
            target_base: Some(10),
            source_number: "1010".to_string(),
            source_alphabet: Some("01".to_string()),
            target_alphabet: Some("0123456789".to_string()),
            config_file: None,
        };

        let config = Config::new(command_line_args);
        let number = BigInt::from(0); // Replace with your number
        let config = &config.unwrap();
        let converter = Converter::new(&config, number);

        assert_eq!(converter.calculate_base(lua_function, 1).unwrap(), BigInt::from(1000000));
    }

    #[test]
    fn test_calculate_base_2() {
        let lua_function = r#"f = function (n) n = BigNum.new(n) base = BigNum.new(10^6) return tostring(base^n) end return f"#;

        let command_line_args = CommandLineArgs {
            source_base: Some(2),
            target_base: Some(10),
            source_number: "1010".to_string(),
            source_alphabet: Some("01".to_string()),
            target_alphabet: Some("0123456789".to_string()),
            config_file: None,
        };

        let config = Config::new(command_line_args);
        let number = BigInt::from(0); // Replace with your number
        let config = &config.unwrap();
        let converter = Converter::new(&config, number);

        assert_eq!(converter.calculate_base(lua_function, 2).unwrap(), BigInt::from_str("1000000000000").unwrap());
    }

    #[test]
    fn test_calculate_base_0() {
        let lua_function = r#"f = function (n) n = BigNum.new(n) base = BigNum.new(10^6) return tostring(base^n) end return f"#;

        let command_line_args = CommandLineArgs {
            source_base: Some(2),
            target_base: Some(10),
            source_number: "1010".to_string(),
            source_alphabet: Some("01".to_string()),
            target_alphabet: Some("0123456789".to_string()),
            config_file: None,
        };

        let config = Config::new(command_line_args);
        let number = BigInt::from(0); // Replace with your number
        let config = &config.unwrap();
        let converter = Converter::new(&config, number);

        assert_eq!(converter.calculate_base(lua_function, 0).unwrap(), BigInt::from(1));
    }

    #[test]
    fn test_factorial_1() {
        let lua_function = r#"f = function (n) n = BigNum.new(n) one = BigNum.new(1) if n <= one then return tostring(one) else return tostring(n * f(n - one)) end end return f"#;

        let command_line_args = CommandLineArgs {
            source_base: Some(2),
            target_base: Some(10),
            source_number: "1".to_string(),
            source_alphabet: Some("01".to_string()),
            target_alphabet: Some("0123456789".to_string()),
            config_file: None,
        };

        let config = Config::new(command_line_args);
        let number = BigInt::from(1);
        let config = &config.unwrap();
        let converter = Converter::new(&config, number);

        assert_eq!(converter.calculate_base(lua_function, 1).unwrap(), BigInt::from(1));
    }

    #[test]
    fn test_factorial_2() {
        let lua_function = r#"f = function (n) n = BigNum.new(n) one = BigNum.new(1) if n <= one then return tostring(one) else return tostring(n * f(n - one)) end end return f"#;

        let command_line_args = CommandLineArgs {
            source_base: Some(2),
            target_base: Some(10),
            source_number: "2".to_string(),
            source_alphabet: Some("01".to_string()),
            target_alphabet: Some("0123456789".to_string()),
            config_file: None,
        };

        let config = Config::new(command_line_args);
        let number = BigInt::from(2);
        let config = &config.unwrap();
        let converter = Converter::new(&config, number);

        assert_eq!(converter.calculate_base(lua_function, 2).unwrap(), BigInt::from(2));
    }

    #[test]
    fn test_factorial_3() {
        let lua_function = r#"f = function (n) n = BigNum.new(n) one = BigNum.new(1) if n <= one then return tostring(one) else return tostring(n * f(n - one)) end end return f"#;

        let command_line_args = CommandLineArgs {
            source_base: Some(2),
            target_base: Some(10),
            source_number: "3".to_string(),
            source_alphabet: Some("01".to_string()),
            target_alphabet: Some("0123456789".to_string()),
            config_file: None,
        };

        let config = Config::new(command_line_args);
        let number = BigInt::from(3);
        let config = &config.unwrap();
        let converter = Converter::new(&config, number);

        assert_eq!(converter.calculate_base(lua_function, 3).unwrap(), BigInt::from(6));
    }
}
