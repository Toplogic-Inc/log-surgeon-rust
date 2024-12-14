use crate::dfa::{State, DFA};
use crate::error_handling::Error::{LexerInputStreamNotSet, LexerInternalErr, LexerStateUnknown};
use crate::error_handling::Result;
use crate::lexer::LexerStream;
use crate::nfa::nfa::NFA;
use crate::parser::ParsedSchema;
use std::collections::VecDeque;
use std::ffi::c_int;
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

pub struct Lexer<'a> {
    schema_mgr: &'a ParsedSchema,
    ts_dfa: DFA,
    var_dfa: DFA,

    state: LexerState,
    dfa_state: State,

    input_stream: Option<Box<dyn LexerStream>>,
    buf: Vec<char>,
    buf_cursor_pos: usize,
    token_queue: VecDeque<Token>,

    last_delimiter: Option<char>,
    last_tokenized_pos: usize,
    match_start_pos: usize,
    match_end_pos: usize,
    line_num: usize,
}

#[derive(Debug)]
enum TokenType {
    Timestamp(usize),
    Variable(usize),
    StaticText,
    StaticTextWithEndLine,
    End,
}

#[derive(Debug)]
pub struct Token {
    val: String,
    token_type: TokenType,
    line_num: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(schema_mgr: &'a ParsedSchema) -> Result<Self> {
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
            schema_mgr,
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
            line_num: 0,
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
        self.line_num = 0;
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
                LexerState::SeekingToTheNextDelimiter => {
                    // TODO: we can skip all non-starting chars
                    let optional_c = self.get_next_char_from_buffer()?;
                    if optional_c.is_none() {
                        self.state = LexerState::EndOfStream;
                        continue;
                    }
                    self.increment_buffer_cursor_pos();
                    let c = optional_c.unwrap();
                    if self.schema_mgr.has_delimiter(c) {
                        self.last_delimiter = Some(c);
                        self.state = LexerState::HandleDelimiter;
                    }
                }

                LexerState::HandleDelimiter => {
                    let delimiter = self.last_delimiter.unwrap();
                    self.last_delimiter = None;
                    if '\n' != delimiter {
                        self.state = LexerState::DFANotAccepted;
                        self.match_start_pos = self.buf_cursor_pos;
                        self.dfa_state = self.var_dfa.get_root();
                        continue;
                    }

                    if self.last_tokenized_pos >= self.buf_cursor_pos {
                        return Err(LexerInternalErr("Delimiter position corrupted"));
                    }
                    self.token_queue.push_back(Token {
                        val: self.buf[self.last_tokenized_pos..self.buf_cursor_pos]
                            .iter()
                            .collect(),
                        line_num: self.line_num,
                        token_type: TokenType::StaticTextWithEndLine,
                    });
                    self.last_tokenized_pos = self.buf_cursor_pos;
                    self.line_num += 1;
                    self.state = LexerState::ParsingTimestamp;
                }

                LexerState::ParsingTimestamp => {
                    self.state = if self.try_parse_timestamp()? {
                        LexerState::SeekingToTheNextDelimiter
                    } else {
                        LexerState::DFANotAccepted
                    }
                }

                LexerState::DFANotAccepted => {
                    let optional_c = self.get_next_char_from_buffer()?;
                    if optional_c.is_none() {
                        self.state = LexerState::EndOfStream;
                        continue;
                    }
                    let c = optional_c.unwrap();
                    if false == c.is_ascii() || self.schema_mgr.has_delimiter(c) {
                        self.state = LexerState::SeekingToTheNextDelimiter;
                        continue;
                    }

                    // NOTE: consume the char only if we're sure it's not delimiter
                    self.increment_buffer_cursor_pos();
                    let optional_next_dfa_state =
                        self.var_dfa.get_next_state(self.dfa_state.clone(), c as u8);
                    if optional_next_dfa_state.is_none() {
                        self.state = LexerState::SeekingToTheNextDelimiter;
                        continue;
                    }

                    self.dfa_state = optional_next_dfa_state.unwrap();
                    match self.var_dfa.is_accept_state(self.dfa_state.clone()) {
                        Some(_) => self.state = LexerState::DFAAccepted,
                        None => self.state = LexerState::DFANotAccepted,
                    }
                }

                LexerState::DFAAccepted => {
                    // Set match end (exclusive to the matched position)
                    self.match_end_pos = self.buf_cursor_pos;

                    let optional_c = self.get_next_char_from_buffer()?;
                    if optional_c.is_none() {
                        self.state = LexerState::VarExtract;
                        continue;
                    }

                    self.increment_buffer_cursor_pos();
                    let c = optional_c.unwrap();
                    if self.schema_mgr.has_delimiter(c) {
                        self.last_delimiter = Some(c);
                        self.state = LexerState::VarExtract;
                        continue;
                    }

                    let optional_next_dfa_state = if c.is_ascii() {
                        self.var_dfa.get_next_state(self.dfa_state.clone(), c as u8)
                    } else {
                        None
                    };
                    if optional_next_dfa_state.is_none() {
                        self.state = LexerState::SeekingToTheNextDelimiter;
                        continue;
                    }

                    self.dfa_state = optional_next_dfa_state.unwrap();
                    match self.var_dfa.is_accept_state(self.dfa_state.clone()) {
                        Some(_) => self.state = LexerState::DFAAccepted,
                        None => self.state = LexerState::DFANotAccepted,
                    }
                }

                LexerState::VarExtract => {
                    // Extract static text
                    if self.match_start_pos > self.last_tokenized_pos {
                        let static_text: String = self.buf
                            [self.last_tokenized_pos..self.match_start_pos]
                            .iter()
                            .collect();
                        self.token_queue.push_back(Token {
                            val: static_text,
                            line_num: self.line_num,
                            token_type: TokenType::StaticText,
                        });
                    }

                    // Extract variable
                    if self.match_start_pos >= self.match_end_pos {
                        return Err(LexerInternalErr("Match positions corrupted"));
                    }
                    let optional_schema_id = self.var_dfa.is_accept_state(self.dfa_state.clone());
                    if optional_schema_id.is_none() {
                        return Err(LexerInternalErr(
                            "DFA state doesn't stop in an accepted state",
                        ));
                    }
                    let schema_id = optional_schema_id.unwrap();
                    self.token_queue.push_back(Token {
                        val: self.buf[self.match_start_pos..self.match_end_pos]
                            .iter()
                            .collect(),
                        line_num: self.line_num,
                        token_type: TokenType::Variable(schema_id),
                    });
                    self.last_tokenized_pos = self.match_end_pos;

                    match self.last_delimiter {
                        Some(_) => self.state = LexerState::HandleDelimiter,
                        None => self.state = LexerState::EndOfStream,
                    }
                }

                LexerState::EndOfStream => {
                    if self.buf_cursor_pos > self.last_tokenized_pos {
                        self.token_queue.push_back(Token {
                            val: self.buf[self.last_tokenized_pos..self.buf_cursor_pos]
                                .iter()
                                .collect(),
                            line_num: self.line_num,
                            token_type: if self.last_delimiter.is_some()
                                && self.last_delimiter.unwrap() == '\n'
                            {
                                // TODO: This seems not possible..
                                TokenType::StaticTextWithEndLine
                            } else {
                                TokenType::StaticText
                            },
                        })
                    }
                    break;
                }
            }

            if false == self.token_queue.is_empty() {
                // TODO: Add garbage collection
                break;
            }
        }

        Ok(())
    }

    fn try_parse_timestamp(&mut self) -> Result<bool> {
        let mut curr_dfa_state = self.ts_dfa.get_root();
        let curr_buf_cursor_pos = self.buf_cursor_pos;

        // (Timestamp schema ID, position)
        let mut last_matched: Option<(usize, usize)> = None;

        loop {
            let optional_c = self.get_next_char_from_buffer()?;
            if optional_c.is_none() {
                break;
            }

            self.increment_buffer_cursor_pos();
            let c = optional_c.unwrap();
            if false == c.is_ascii() {
                break;
            }

            let optional_next_state = self.ts_dfa.get_next_state(curr_dfa_state.clone(), c as u8);
            if optional_next_state.is_none() {
                break;
            }
            curr_dfa_state = optional_next_state.unwrap();

            match self.ts_dfa.is_accept_state(self.dfa_state.clone()) {
                Some(ts_schema_id) => last_matched = Some((ts_schema_id, self.buf_cursor_pos)),
                None => {}
            }
        }

        match last_matched {
            Some((ts_schema_id, pos)) => {
                self.token_queue.push_back(Token {
                    val: self.buf[curr_buf_cursor_pos..pos].iter().collect(),
                    line_num: self.line_num,
                    token_type: TokenType::Timestamp(ts_schema_id),
                });
                self.last_tokenized_pos = pos;
                self.buf_cursor_pos = pos;
                Ok(true)
            }
            None => {
                self.buf_cursor_pos = curr_buf_cursor_pos;
                Ok(false)
            }
        }
    }

    fn get_next_char_from_buffer(&mut self) -> Result<Option<char>> {
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
        Ok(Some(self.buf[self.buf_cursor_pos]))
    }

    fn increment_buffer_cursor_pos(&mut self) {
        self.buf_cursor_pos += 1
    }
}
