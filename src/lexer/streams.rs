use super::lexer_stream::LexerStream;
use crate::error_handling::Error::IOError;
use crate::error_handling::Result;
use std::io::BufRead;

pub struct BufferedFileStream {
    line_it: std::io::Lines<std::io::BufReader<std::fs::File>>,
    line: Option<Vec<char>>,
    pos: usize,
}

impl BufferedFileStream {
    pub fn new(path: &str) -> Result<Self> {
        match std::fs::File::open(path) {
            Ok(file) => Ok(Self {
                line_it: std::io::BufReader::new(file).lines(),
                line: None,
                pos: 0,
            }),
            Err(e) => Err(IOError(e)),
        }
    }
}

impl LexerStream for BufferedFileStream {
    fn get_next_char(&mut self) -> Result<Option<char>> {
        if self.line.is_none() {
            let next_line = self.line_it.next();
            if next_line.is_none() {
                return Ok(None);
            }
            match next_line.unwrap() {
                Ok(line) => {
                    self.line = Some(line.chars().collect());
                    self.line.as_mut().unwrap().push('\n');
                    self.pos = 0;
                }
                Err(e) => return Err(IOError(e)),
            }
        }

        let c = self.line.as_ref().unwrap()[self.pos];
        self.pos += 1;
        if self.pos == self.line.as_ref().unwrap().len() {
            self.line = None;
        }
        Ok(Some(c))
    }
}
