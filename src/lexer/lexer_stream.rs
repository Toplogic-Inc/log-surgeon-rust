use crate::error_handling::Result;

pub trait LexerStream {
    fn get_next_char(&mut self) -> Result<Option<char>>;
}
