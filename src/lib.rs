pub mod error_handling;
pub mod lexer;
pub mod log_parser;
pub mod parser;

#[cfg(feature = "regex-engine")]
pub mod dfa;
#[cfg(feature = "regex-engine")]
pub mod nfa;

#[cfg(not(feature = "regex-engine"))]
mod dfa;
#[cfg(not(feature = "regex-engine"))]
mod nfa;

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
