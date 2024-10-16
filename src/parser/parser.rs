use super::ast_node::ASTNode;
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
        self.parse_union()
    }

    // Deal with union (symbol '|')
    fn parse_union(&mut self) -> Option<ASTNode> {
        let mut node = self.parse_concat()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Union => {
                    self.next();
                    let right = self.parse_concat()?;
                    node = ASTNode::Union(Box::new(node), Box::new(right));
                }
                _ => {
                    break;
                }
            };
        }

        Some(node)
    }

    // deal with concatenation
    fn parse_concat(&mut self) -> Option<ASTNode> {
        let mut node = self.parse_repetition()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Literal(_) | Token::LParen => {
                    let right = self.parse_repetition()?;
                    node = ASTNode::Concat(Box::new(node), Box::new(right));
                }
                _ => break,
            }
        }

        Some(node)
    }

    // Deal with * + ? repetition
    fn parse_repetition(&mut self) -> Option<ASTNode> {
        let mut node = self.parse_base()?;

        match self.peek() {
            Some(Token::Star) => {
                self.next();
                node = ASTNode::Star(Box::new(node));
            }
            Some(Token::Plus) => {
                self.next();
                node = ASTNode::Plus(Box::new(node));
            }
            Some(Token::Optional) => {
                self.next();
                node = ASTNode::Optional(Box::new(node));
            }
            _ => {}
        }

        Some(node)
    }

    // parse literal, or group
    fn parse_base(&mut self) -> Option<ASTNode> {
        match self.next()? {
            Token::Literal(l) => Some(ASTNode::Literal(*l)),
            Token::LParen => {
                let expr = self.parse_regex()?;
                match self.next()? {
                    Token::RParen => Some(ASTNode::Group(Box::new(expr))),
                    _ => {
                        println!("Expected closing parenthesis");
                        None
                    }
                }
            }
            _ => None,
        }
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

    #[test]
    fn test_basic_union_regex_to_ast() {
        let mut p = ParserStream::new("a|b");
        let ast = p.parse_regex();

        assert_eq!(
            ast,
            Some(ASTNode::Union(
                Box::new(ASTNode::Literal('a')),
                Box::new(ASTNode::Literal('b'))
            ))
        );
    }

    #[test]
    fn test_basic_concat_regex_to_ast() {
        let mut p = ParserStream::new("ab");
        let ast = p.parse_regex();

        assert_eq!(
            ast,
            Some(ASTNode::Concat(
                Box::new(ASTNode::Literal('a')),
                Box::new(ASTNode::Literal('b'))
            ))
        );
    }

    #[test]
    fn test_basic_repetition_regex_to_ast() {
        let mut p = ParserStream::new("a*");
        let ast = p.parse_regex();

        assert_eq!(ast, Some(ASTNode::Star(Box::new(ASTNode::Literal('a')))));
    }
}
