use super::ast_node::ast_node::ASTNode;
use super::token::Token;

pub struct ParserStream {
    tokens: Vec<Token>,
    pos: usize, // Current position in the token stream
}

impl ParserStream {
    pub fn new(regex: &str) -> Self {
        let tokens = Token::tokenize(regex);
        ParserStream { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn get_token(&self, pos: usize) -> Option<&Token> {
        self.tokens.get(pos)
    }
}

impl ParserStream {
    fn parse_regex(&mut self) -> Option<ASTNode> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_tokenization() {
        let p = ParserStream::new("a|(b*)c?de+f");
        assert!(p.get_token(0) == Some(&Token::Literal('a')));
        assert!(p.get_token(1) == Some(&Token::Union));
        assert!(p.get_token(2) == Some(&Token::LParen));
        assert!(p.get_token(3) == Some(&Token::Literal('b')));
        assert!(p.get_token(4) == Some(&Token::Star));
        assert!(p.get_token(5) == Some(&Token::RParen));
        assert!(p.get_token(6) == Some(&Token::Literal('c')));
        assert!(p.get_token(7) == Some(&Token::Optional));
        assert!(p.get_token(8) == Some(&Token::Literal('d')));
        assert!(p.get_token(9) == Some(&Token::Literal('e')));
        assert!(p.get_token(10) == Some(&Token::Plus));
        assert!(p.get_token(11) == Some(&Token::Literal('f')));
    }
}
