#[derive(PartialEq)]
pub(crate) enum Token {
    Literal(char),  // Single character
    Star,           // *
    Plus,           // +
    Optional,       // ?
    Union,          // |
    LParen,         // (
    RParen,         // )
}

impl Token {
    pub(crate) fn tokenize(regex: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        for ch in regex.chars() {
            match ch {
                '*' => tokens.push(Token::Star),
                '+' => tokens.push(Token::Plus),
                '?' => tokens.push(Token::Optional),
                '|' => tokens.push(Token::Union),
                '(' => tokens.push(Token::LParen),
                ')' => tokens.push(Token::RParen),
                _   => tokens.push(Token::Literal(ch)),  // All other characters are literals
            }
        }
        tokens
    }
}
