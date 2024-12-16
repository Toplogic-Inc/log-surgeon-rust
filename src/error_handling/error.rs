use regex_syntax::ast;

#[derive(Debug)]
pub enum Error {
    RegexParsingError(ast::Error),
    YamlParsingError(serde_yaml::Error),
    IOError(std::io::Error),
    UnsupportedAstNodeType(&'static str),
    NoneASCIICharacters,
    NegationNotSupported(&'static str),
    NonGreedyRepetitionNotSupported,
    UnsupportedAstBracketedKind,
    UnsupportedClassSetType,
    UnsupportedGroupKindType,
    MissingSchemaKey(&'static str),
    LexerInputStreamNotSet,
    LexerStateUnknown,
    LexerInternalErr(&'static str),
    LogParserInternalErr(&'static str),
    InvalidSchema,
}

pub type Result<T> = std::result::Result<T, Error>;
