use regex_syntax::ast;

#[derive(Debug)]
pub enum Error {
    RegexParsingError(ast::Error),
    UnsupportedAstNodeType(&'static str),
    NoneASCIICharacters,
    NegationNotSupported(&'static str),
    NonGreedyRepetitionNotSupported,
    UnsupportedAstBracketedKind,
    UnsupportedClassSetType,
    UnsupportedGroupKindType,
}

pub type Result<T> = std::result::Result<T, Error>;
