use regex_syntax::ast;

#[derive(Debug)]
pub enum Error {
    RegexParsingError(ast::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
