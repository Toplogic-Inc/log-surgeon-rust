use super::lexer_stream::LexerStream;
use crate::error_handling::Error::IOError;
use crate::error_handling::Result;
use std::io::{self, BufReader, Read};

const BUF_SIZE: usize = 4096 * 16;

pub struct BufferedFileStream {
    buf_reader: BufReader<std::fs::File>,
    pos: usize,
    end: usize,
    buffer: [u8; 4096],
}

impl BufferedFileStream {
    pub fn new(path: &str) -> Result<Self> {
        match std::fs::File::open(path) {
            Ok(file) => Ok(Self {
                buf_reader: BufReader::new(file),
                pos: 0,
                end: 0,
                buffer: [0; 4096],
            }),
            Err(e) => Err(IOError(e)),
        }
    }
}

impl LexerStream for BufferedFileStream {
    fn get_next_char(&mut self) -> Result<Option<u8>> {
        if self.pos == self.end {
            match self.buf_reader.read(&mut self.buffer) {
                Ok(byte_read) => {
                    if 0 == byte_read {
                        return Ok(None);
                    }
                    self.end = byte_read;
                    self.pos = 0;
                }
                Err(e) => return Err(IOError(e)),
            }
        }
        let c = self.buffer[self.pos];
        self.pos += 1;
        Ok(Some(c))
    }
}
