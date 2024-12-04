use crate::error_handling::{Error, Error::RegexParsingError, Result};
use regex_syntax::ast::{parse::Parser, Ast};

// This is a wrapper of `regex_syntax::ast::parse::Parser`, which can be extended to hold
// program-specific data members.
pub struct RegexParser {
    m_parser: Parser,
}

impl RegexParser {
    pub fn new() -> RegexParser {
        Self {
            m_parser: Parser::new(),
        }
    }

    pub fn parse_into_ast(&mut self, pattern: &str) -> Result<Ast> {
        match self.m_parser.parse(pattern) {
            Ok(ast) => Ok(ast),
            Err(e) => Err(RegexParsingError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex_syntax::ast;

    #[test]
    fn test_basic_parsing() {
        let mut parser = RegexParser::new();
        let parse_result = parser.parse_into_ast(r"[a-t\d]");
        assert!(parse_result.is_ok());
        let Ast::ClassBracketed(bracket_ast) = &parse_result.unwrap() else {
            panic!("Type mismatched")
        };
        let ast::ClassSet::Item(item) = &bracket_ast.kind else {
            panic!("Type mismatched")
        };
        let ast::ClassSetItem::Union(union) = &item else {
            panic!("Type mismatched")
        };
        let a_to_z_item = &union.items[0];
        let ast::ClassSetItem::Range(range) = &a_to_z_item else {
            panic!("Type mismatched")
        };
        assert_eq!(range.start.c, 'a');
        assert_eq!(range.end.c, 't');
        let digit_item = &union.items[1];
        let ast::ClassSetItem::Perl(perl) = &digit_item else {
            panic!("Type mismatched")
        };
        let ast::ClassPerlKind::Digit = &perl.kind else {
            panic!("Type mismatched")
        };
    }
}
