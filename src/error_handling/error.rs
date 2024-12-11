use regex_syntax::ast;

#[derive(Debug)]
pub enum Error {
    RegexParsingError(ast::Error),
    AstToNfaNotSupported(&'static str),
    NoneASCIICharacters,
    NegatedPerl,
}

pub type Result<T> = std::result::Result<T, Error>;
