use crate::dfa::dfa::{State, DFA};
use crate::error_handling::Error::{LexerInputStreamNotSet, LexerInternalErr, LexerStateUnknown};
use crate::error_handling::Result;
use crate::lexer::LexerStream;
use crate::nfa::nfa::NFA;
use crate::parser::SchemaConfig;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::rc::Rc;

enum LexerState {
    SeekingToTheNextDelimiter,
    HandleDelimiter,
    DFANotAccepted,
    DFAAccepted,
    VarExtract,
    ParsingTimestamp,
    EndOfStream,
}

pub struct Lexer {
    schema_config: Rc<SchemaConfig>,
    ts_dfa: DFA,
    var_dfa: DFA,

    state: LexerState,
    dfa_state: State,

    input_stream: Option<Box<dyn LexerStream>>,
    buf: Vec<u8>,
    buf_cursor_pos: usize,
    token_queue: VecDeque<Token>,

    last_delimiter: Option<u8>,
    last_tokenized_pos: usize,
    match_start_pos: usize,
    match_end_pos: usize,
    line_num: usize,
}

#[derive(Clone, Debug)]
pub enum TokenType {
    Timestamp(usize),
    Variable(usize),
    StaticText,
    StaticTextWithEndLine,
    End,
}

pub struct Token {
    buf: Vec<u8>,
    token_type: TokenType,
    line_num: usize,
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{:?}|{}]: \"{}\"",
            self.token_type,
            self.line_num,
            self.get_buf_as_string().escape_default()
        )
    }
}

impl Token {
    pub fn get_buf(&self) -> &[u8] {
        self.buf.as_slice()
    }

    pub fn get_buf_as_string(&self) -> String {
        String::from_utf8_lossy(&self.buf).to_string()
    }

    pub fn get_token_type(&self) -> TokenType {
        self.token_type.clone()
    }

    pub fn get_line_num(&self) -> usize {
        self.line_num
    }
}

impl Lexer {
    const MIN_BUF_GARBAGE_COLLECTION_SIZE: usize = 4096;

    pub fn new(schema_mgr: Rc<SchemaConfig>) -> Result<Self> {
        let mut ts_nfas: Vec<NFA> = Vec::new();
        for schema in schema_mgr.get_ts_schemas() {
            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(schema.get_ast(), nfa.get_start(), nfa.get_accept())?;
            ts_nfas.push(nfa);
        }
        let ts_dfa = DFA::from_multiple_nfas(ts_nfas);

        let mut var_nfas: Vec<NFA> = Vec::new();
        for schema in schema_mgr.get_var_schemas() {
            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(schema.get_ast(), nfa.get_start(), nfa.get_accept())?;
            var_nfas.push(nfa);
        }
        let var_dfa = DFA::from_multiple_nfas(var_nfas);
        let var_dfa_root = var_dfa.get_root();

        Ok(Self {
            schema_config: schema_mgr,
            ts_dfa,
            var_dfa,
            state: LexerState::ParsingTimestamp,
            dfa_state: var_dfa_root,
            input_stream: None,
            buf: Vec::new(),
            buf_cursor_pos: 0,
            token_queue: VecDeque::new(),
            last_delimiter: None,
            last_tokenized_pos: 0,
            match_start_pos: 0,
            match_end_pos: 0,
            line_num: 1,
        })
    }

    fn reset(&mut self) {
        self.input_stream = None;
        self.buf.clear();
        self.buf_cursor_pos = 0;
        self.token_queue.clear();
        self.last_delimiter = None;
        self.last_tokenized_pos = 0;
        self.match_start_pos = 0;
        self.match_end_pos = 0;
        self.line_num = 1;
        self.state = LexerState::ParsingTimestamp;
    }

    pub fn set_input_stream(&mut self, input_stream: Box<dyn LexerStream>) {
        self.reset();
        self.input_stream = Some(input_stream);
        self.state = LexerState::ParsingTimestamp;
    }

    pub fn get_next_token(&mut self) -> Result<Option<Token>> {
        if self.input_stream.is_none() {
            return Err(LexerInputStreamNotSet);
        }
        if self.token_queue.is_empty() {
            self.fill_token_queue()?;
        }
        Ok(self.token_queue.pop_front())
    }

    fn fill_token_queue(&mut self) -> Result<()> {
        loop {
            match self.state {
                LexerState::SeekingToTheNextDelimiter => match self.get_next_char_from_buffer()? {
                    Some(c) => {
                        if self.schema_config.has_delimiter(c) {
                            self.last_delimiter = Some(c);
                            self.state = LexerState::HandleDelimiter;
                        }
                    }
                    None => {
                        self.state = LexerState::EndOfStream;
                    }
                },

                LexerState::HandleDelimiter => {
                    if self.last_delimiter.is_none() {
                        return Err(LexerInternalErr("Delimiter not set"));
                    }

                    let delimiter = self.last_delimiter.unwrap();
                    self.last_delimiter = None;
                    match delimiter {
                        b'\n' => {
                            self.generate_token(
                                self.buf_cursor_pos,
                                TokenType::StaticTextWithEndLine,
                            )?;
                            self.line_num += 1;
                            self.state = LexerState::ParsingTimestamp;
                        }
                        _ => self.proceed_to_var_dfa_simulation(),
                    }
                }

                LexerState::ParsingTimestamp => {
                    if self.try_parse_timestamp()? {
                        self.state = LexerState::SeekingToTheNextDelimiter;
                    } else {
                        self.proceed_to_var_dfa_simulation();
                    }
                }

                LexerState::DFANotAccepted => match self.get_next_char_from_buffer()? {
                    Some(c) => {
                        self.simulate_var_dfa_and_set_lexer_state(c, LexerState::HandleDelimiter)
                    }
                    None => self.state = LexerState::EndOfStream,
                },

                LexerState::DFAAccepted => {
                    // Set match end (exclusive to the matched position)
                    self.match_end_pos = self.buf_cursor_pos;
                    match self.get_next_char_from_buffer()? {
                        Some(c) => {
                            self.simulate_var_dfa_and_set_lexer_state(c, LexerState::VarExtract)
                        }
                        None => self.state = LexerState::VarExtract,
                    }
                }

                LexerState::VarExtract => {
                    if self.match_start_pos >= self.match_end_pos {
                        return Err(LexerInternalErr("Match end positions corrupted"));
                    }
                    if self.last_tokenized_pos > self.buf_cursor_pos {
                        return Err(LexerInternalErr("Match start position corrupted"));
                    }

                    // Extract static text (if any)
                    if self.match_start_pos != self.last_tokenized_pos {
                        self.generate_token(self.match_start_pos, TokenType::StaticText)?;
                    }

                    // Extract variable
                    match self.var_dfa.is_accept_state(self.dfa_state.clone()) {
                        Some(schema_id) => {
                            assert_eq!(self.match_start_pos, self.last_tokenized_pos);
                            self.generate_token(
                                self.match_end_pos,
                                TokenType::Variable(schema_id),
                            )?;
                        }
                        None => {
                            return Err(LexerInternalErr(
                                "DFA state doesn't stop in an accepted state",
                            ))
                        }
                    }

                    match self.last_delimiter {
                        Some(_) => self.state = LexerState::HandleDelimiter,
                        None => self.state = LexerState::EndOfStream,
                    }
                }

                LexerState::EndOfStream => {
                    if self.buf_cursor_pos > self.last_tokenized_pos {
                        let token_type = if self.last_delimiter.is_some()
                            && self.last_delimiter.unwrap() == b'\n'
                        {
                            // TODO: This seems not possible..
                            TokenType::StaticTextWithEndLine
                        } else {
                            TokenType::StaticText
                        };
                        self.generate_token(self.buf_cursor_pos, token_type)?;
                    }
                    break;
                }
            }

            if false == self.token_queue.is_empty() {
                break;
            }
        }

        self.buffer_garbage_collection();
        Ok(())
    }

    fn try_parse_timestamp(&mut self) -> Result<bool> {
        let buf_cursor_pos_bookmark = self.buf_cursor_pos;
        if buf_cursor_pos_bookmark != self.last_tokenized_pos {
            return Err(LexerInternalErr("Timestamp parsing corrupted"));
        }
        let mut curr_dfa_state = self.ts_dfa.get_root();

        // (Timestamp schema ID, position)
        let mut last_matched: Option<(usize, usize)> = None;

        loop {
            let optional_c = self.get_next_char_from_buffer()?;
            if optional_c.is_none() {
                break;
            }

            let c = optional_c.unwrap();
            if false == c.is_ascii() {
                break;
            }

            let optional_next_state = self.ts_dfa.get_next_state(curr_dfa_state.clone(), c as u8);
            if optional_next_state.is_none() {
                break;
            }
            curr_dfa_state = optional_next_state.unwrap();

            match self.ts_dfa.is_accept_state(curr_dfa_state.clone()) {
                Some(ts_schema_id) => last_matched = Some((ts_schema_id, self.buf_cursor_pos)),
                None => {}
            }
        }

        match last_matched {
            Some((ts_schema_id, pos)) => {
                self.generate_token(pos, TokenType::Timestamp(ts_schema_id))?;
                self.buf_cursor_pos = pos;
                Ok(true)
            }
            None => {
                self.buf_cursor_pos = buf_cursor_pos_bookmark;
                Ok(false)
            }
        }
    }

    fn get_next_char_from_buffer(&mut self) -> Result<Option<u8>> {
        let pos = self.buf_cursor_pos;
        if pos == self.buf.len() {
            match self
                .input_stream
                .as_mut()
                .unwrap()
                .as_mut()
                .get_next_char()?
            {
                Some(c) => self.buf.push(c),
                None => return Ok(None),
            }
        }
        let pos = self.get_and_increment_buf_cursor_pos();
        Ok(Some(self.buf[pos]))
    }

    fn capture_delimiter(&mut self, c: u8) -> bool {
        if self.schema_config.has_delimiter(c) {
            self.last_delimiter = Some(c);
            return true;
        }
        false
    }

    fn simulate_var_dfa_and_set_lexer_state(&mut self, c: u8, delimiter_dst_state: LexerState) {
        match self.var_dfa.get_next_state(self.dfa_state.clone(), c) {
            Some(next_dfa_state) => {
                self.dfa_state = next_dfa_state;
                match self.var_dfa.is_accept_state(self.dfa_state.clone()) {
                    Some(_) => self.state = LexerState::DFAAccepted,
                    None => self.state = LexerState::DFANotAccepted,
                }
            }
            None => {
                self.state = if self.capture_delimiter(c) {
                    delimiter_dst_state
                } else {
                    LexerState::SeekingToTheNextDelimiter
                };
            }
        }
    }

    fn proceed_to_var_dfa_simulation(&mut self) {
        self.match_start_pos = self.buf_cursor_pos;
        self.dfa_state = self.var_dfa.get_root();
        self.state = LexerState::DFANotAccepted;
    }

    fn generate_token(&mut self, end_pos: usize, token_type: TokenType) -> Result<()> {
        if end_pos <= self.last_tokenized_pos {
            return Err(LexerInternalErr("Tokenization end position corrupted"));
        }
        self.token_queue.push_back(Token {
            buf: self.buf[self.last_tokenized_pos..end_pos]
                .iter()
                .map(|c| c.clone())
                .collect(),
            line_num: self.line_num,
            token_type,
        });
        self.last_tokenized_pos = end_pos;
        Ok(())
    }

    fn get_and_increment_buf_cursor_pos(&mut self) -> usize {
        let curr_pos = self.buf_cursor_pos;
        self.buf_cursor_pos += 1;
        curr_pos
    }

    fn set_buf_cursor_pos(&mut self, pos: usize) {
        self.buf_cursor_pos = pos;
    }

    fn buffer_garbage_collection(&mut self) {
        if self.last_tokenized_pos <= self.buf.len() / 2
            || self.last_tokenized_pos <= Self::MIN_BUF_GARBAGE_COLLECTION_SIZE
        {
            return;
        }

        let mut dst_idx = 0usize;
        let mut src_idx = self.last_tokenized_pos;
        while src_idx < self.buf.len() {
            self.buf[dst_idx] = self.buf[src_idx];
            dst_idx += 1;
            src_idx += 1;
        }
        self.buf.resize(dst_idx, 0);
        self.buf_cursor_pos -= self.last_tokenized_pos;
        self.last_tokenized_pos = 0;
        // No need to reset match_start/end
    }
}
