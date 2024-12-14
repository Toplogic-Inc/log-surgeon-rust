mod dfa;
pub mod error_handling;
pub mod lexer;
pub mod log_parser;
mod nfa;
pub mod parser;

const VERSION: &str = "0.0.1";

pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), VERSION);
    }
}
