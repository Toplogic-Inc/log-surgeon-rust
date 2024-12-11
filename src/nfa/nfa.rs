use crate::error_handling::Error;
use crate::error_handling::Result;
use crate::parser::regex_parser::parser::RegexParser;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use crate::error_handling::Error::{AstToNfaNotSupported, NegatedPerl, NoneASCIICharacters};
use crate::parser::ast_node::ast_node::AstNode;
use crate::parser::ast_node::ast_node_concat::AstNodeConcat;
use crate::parser::ast_node::ast_node_literal::AstNodeLiteral;
use crate::parser::ast_node::ast_node_optional::AstNodeOptional;
use crate::parser::ast_node::ast_node_plus::AstNodePlus;
use crate::parser::ast_node::ast_node_star::AstNodeStar;
use crate::parser::ast_node::ast_node_union::AstNodeUnion;
use regex_syntax::ast::{Ast, ClassPerl, ClassPerlKind, Literal};

const DIGIT_TRANSITION: u128 = 0x000000000000000003ff000000000000;
const SPACE_TRANSITION: u128 = 0x00000000000000000000000100003e00;
const WORD_TRANSITION: u128 = 0x07fffffe87fffffe03ff000000000000;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct State(pub usize);

pub struct Transition {
    from: State,
    to: State,
    symbol_onehot_encoding: u128,
    tag: i16,
}

impl Debug for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:?} -> {:?}, symbol: {:?}",
            self.from, self.to, self.symbol_onehot_encoding
        )
    }
}

impl Transition {
    pub fn convert_char_to_symbol_onehot_encoding(c: char) -> u128 {
        let mut symbol_onehot_encoding = 0;
        let c = c as u8;

        symbol_onehot_encoding |= 1 << c;

        symbol_onehot_encoding
    }

    pub fn convert_char_range_to_symbol_onehot_encoding(range: Option<(u8, u8)>) -> u128 {
        let mut symbol_onehot_encoding: u128 = 0;

        match range {
            Some((begin, end)) => {
                for c in begin..=end {
                    symbol_onehot_encoding |= 1 << c;
                }
            }
            None => {}
        }

        symbol_onehot_encoding
    }

    pub fn convert_char_vec_to_symbol_onehot_encoding(char_vec: Vec<u8>) -> u128 {
        let mut symbol_onehot_encoding: u128 = 0;
        for c in char_vec {
            symbol_onehot_encoding |= 1 << c;
        }
        symbol_onehot_encoding
    }

    pub fn new(from: State, to: State, symbol_onehot_encoding: u128, tag: i16) -> Self {
        Transition {
            from,
            to,
            symbol_onehot_encoding,
            tag,
        }
    }

    pub fn get_symbol_onehot_encoding(&self) -> u128 {
        self.symbol_onehot_encoding
    }

    pub fn get_symbol(&self) -> Vec<char> {
        let mut symbol = vec![];
        for i in 0..=127 {
            if self.symbol_onehot_encoding & (1 << i) != 0 {
                symbol.push(i as u8 as char);
            }
        }
        symbol
    }

    pub fn get_to_state(&self) -> State {
        self.to.clone()
    }
}

pub(crate) struct NFA {
    start: State,
    accept: State,
    states: Vec<State>,
    transitions: HashMap<State, Vec<Transition>>,
}

// NFA implementation for NFA construction from AST
impl NFA {
    pub fn new() -> Self {
        let start = State(0);
        let accept = State(1);
        let states_vec = vec![start.clone(), accept.clone()];
        NFA {
            start,
            accept,
            states: states_vec,
            transitions: HashMap::new(),
        }
    }

    pub fn add_ast_to_nfa(&mut self, ast: &Ast, start: State, end: State) -> Result<()> {
        match ast {
            Ast::Literal(literal) => self.add_literal(&**literal, start, end)?,
            Ast::ClassPerl(perl) => self.add_perl(&**perl, start, end)?,
            Ast::Concat(concat) => {
                let mut curr_start = start.clone();
                for (idx, sub_ast) in concat.asts.iter().enumerate() {
                    let curr_end = if concat.asts.len() - 1 == idx {
                        end.clone()
                    } else {
                        self.new_state()
                    };
                    self.add_ast_to_nfa(sub_ast, curr_start.clone(), curr_end.clone())?;
                    curr_start = curr_end.clone();
                }
            }
            _ => {
                return Err(AstToNfaNotSupported("Ast Type not supported"));
            }
        }
        Ok(())
    }

    fn add_literal(&mut self, literal: &Literal, start: State, end: State) -> Result<()> {
        let c = get_ascii_char(literal.c)?;
        self.add_transition_from_range(start, end, Some((c, c)));
        Ok(())
    }

    fn add_perl(&mut self, perl: &ClassPerl, start: State, end: State) -> Result<()> {
        if perl.negated {
            return Err(NegatedPerl);
        }
        match perl.kind {
            ClassPerlKind::Digit => self.add_transition(start, end, DIGIT_TRANSITION),
            ClassPerlKind::Space => self.add_transition(start, end, SPACE_TRANSITION),
            ClassPerlKind::Word => self.add_transition(start, end, WORD_TRANSITION),
        }
        Ok(())
    }

    fn new_state(&mut self) -> State {
        self.states.push(State(self.states.len()));
        self.states.last().unwrap().clone()
    }

    fn add_transition_from_range(&mut self, from: State, to: State, range: Option<(u8, u8)>) {
        let transition = Transition {
            from: from.clone(),
            to: to.clone(),
            symbol_onehot_encoding: Transition::convert_char_range_to_symbol_onehot_encoding(range),
            tag: -1,
        };
        self.transitions
            .entry(from)
            .or_insert(vec![])
            .push(transition);
    }

    fn add_transition(&mut self, from: State, to: State, onehot: u128) {
        let transition = Transition {
            from: from.clone(),
            to: to.clone(),
            symbol_onehot_encoding: onehot,
            tag: -1,
        };
        self.transitions
            .entry(from)
            .or_insert(vec![])
            .push(transition);
    }

    fn add_epsilon_transition(&mut self, from: State, to: State) {
        self.add_transition(from, to, 0);
    }

    // fn add_state(&mut self, state: State) {
    //     self.states.insert(state);
    // }
    //
    // fn add_transition(&mut self, transition: Transition) {
    //     self.transitions
    //         .entry(transition.from.clone())
    //         .or_insert(vec![])
    //         .push(transition);
    // }
    //
    // }
    //
    // // Offset all states by a given amount
    // fn offset_states(&mut self, offset: usize) {
    //     if offset == 0 {
    //         return;
    //     }
    //
    //     // Update start and accept states
    //     self.start = State(self.start.0 + offset);
    //     self.accept = State(self.accept.0 + offset);
    //
    //     // Update all states
    //     let mut new_states = HashSet::new();
    //     for state in self.states.iter() {
    //         new_states.insert(State(state.0 + offset));
    //     }
    //     self.states = new_states;
    //
    //     // Update transitions in place by adding the offset to each state's "from" and "to" values
    //     let mut updated_transitions: HashMap<State, Vec<Transition>> = HashMap::new();
    //     for (start, transitions) in self.transitions.iter() {
    //         let updated_start = State(start.0 + offset);
    //         let updated_transitions_list: Vec<Transition> = transitions
    //             .iter()
    //             .map(|transition| Transition {
    //                 from: State(transition.from.0 + offset),
    //                 to: State(transition.to.0 + offset),
    //                 symbol_onehot_encoding: transition.symbol_onehot_encoding,
    //                 tag: transition.tag,
    //             })
    //             .collect();
    //         updated_transitions.insert(updated_start, updated_transitions_list);
    //     }
    //
    //     self.transitions = updated_transitions;
    // }
}

impl Debug for NFA {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "NFA( start: {:?}, accept: {:?}, states: {:?}, transitions: {{\n",
            self.start, self.accept, self.states
        )?;
        for (state, transitions) in &self.transitions {
            write!(f, "\t{:?}:\n", state)?;
            for transition in transitions {
                write!(f, "\t\t{:?}\n", transition)?;
            }
        }
        write!(f, "}} )")
    }
}

// NFA implementation for NFA to dfa conversion helper functions
impl NFA {
    pub fn epsilon_closure(&self, states: &Vec<State>) -> Vec<State> {
        let mut closure = states.clone();
        let mut stack = states.clone();

        while let Some(state) = stack.pop() {
            let transitions = self.transitions.get(&state);
            if transitions.is_none() {
                continue;
            }

            for transition in transitions.unwrap() {
                if transition.symbol_onehot_encoding == 0 {
                    let to_state = transition.to.clone();
                    if !closure.contains(&to_state) {
                        closure.push(to_state.clone());
                        stack.push(to_state);
                    }
                }
            }
        }

        closure
    }

    // Static function to get the combined state names
    pub fn get_combined_state_names(states: &Vec<State>) -> String {
        let mut names = states
            .iter()
            .map(|state| state.0.to_string())
            .collect::<Vec<String>>();
        names.sort();
        names.join(",")
    }
}

// Getter functions for NFA
impl NFA {
    pub fn get_start(&self) -> State {
        self.start.clone()
    }

    pub fn get_accept(&self) -> State {
        self.accept.clone()
    }

    pub fn get_transitions(&self) -> &HashMap<State, Vec<Transition>> {
        &self.transitions
    }

    pub fn get_transitions_from_state(&self, state: &State) -> Option<&Vec<Transition>> {
        self.transitions.get(state)
    }
}

// Helper functions
fn get_ascii_char(c: char) -> Result<u8> {
    if false == c.is_ascii() {
        return Err(NoneASCIICharacters);
    }
    Ok(c as u8)
}

// Test use only functions for DFA

#[cfg(test)]
impl NFA {
    // pub fn test_extern_add_state(&mut self, state: State) {
    //     self.add_state(state);
    // }
    //
    // pub fn test_extern_add_transition(&mut self, transition: Transition) {
    //     self.add_transition(transition);
    // }
    //
    // pub fn test_extern_add_epsilon_transition(&mut self, from: State, to: State) {
    //     self.add_epsilon_transition(from, to);
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_char() -> Result<()> {
        let mut parser = RegexParser::new();
        let parsed_ast = parser.parse_into_ast(r"&")?;
        let mut nfa = NFA::new();
        nfa.add_ast_to_nfa(&parsed_ast, State(0), State(1))?;
        assert_eq!(nfa.transitions.len(), 1);

        let start_transitions = nfa.transitions.get(&State(0)).unwrap();
        assert_eq!(start_transitions.len(), 1);
        assert_eq!(start_transitions[0].from, State(0));
        assert_eq!(start_transitions[0].to, State(1));
        assert_eq!(
            start_transitions[0].symbol_onehot_encoding,
            Transition::convert_char_to_symbol_onehot_encoding('&')
        );
        Ok(())
    }

    #[test]
    fn test_perl() -> Result<()> {
        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"\d")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, State(0), State(1))?;
            assert_eq!(nfa.transitions.len(), 1);

            let start_transitions = nfa.transitions.get(&State(0)).unwrap();
            assert_eq!(start_transitions.len(), 1);
            assert_eq!(start_transitions[0].from, State(0));
            assert_eq!(start_transitions[0].to, State(1));

            let char_vec: Vec<u8> = (b'0'..=b'9').collect();
            assert_eq!(
                start_transitions[0].symbol_onehot_encoding,
                Transition::convert_char_vec_to_symbol_onehot_encoding(char_vec)
            );
        }

        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"\s")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, State(0), State(1))?;
            assert_eq!(nfa.transitions.len(), 1);

            let start_transitions = nfa.transitions.get(&State(0)).unwrap();
            assert_eq!(start_transitions.len(), 1);
            assert_eq!(start_transitions[0].from, State(0));
            assert_eq!(start_transitions[0].to, State(1));

            let char_vec = vec![
                b' ',    // Space
                b'\t',   // Horizontal Tab
                b'\n',   // Line Feed
                b'\r',   // Carriage Return
                b'\x0B', // Vertical Tab
                b'\x0C', // Form Feed
            ];
            assert_eq!(
                start_transitions[0].symbol_onehot_encoding,
                Transition::convert_char_vec_to_symbol_onehot_encoding(char_vec)
            );
        }

        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"\w")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, State(0), State(1))?;
            assert_eq!(nfa.transitions.len(), 1);

            let start_transitions = nfa.transitions.get(&State(0)).unwrap();
            assert_eq!(start_transitions.len(), 1);
            assert_eq!(start_transitions[0].from, State(0));
            assert_eq!(start_transitions[0].to, State(1));

            let char_vec: Vec<u8> = (b'0'..=b'9')
                .chain(b'A'..=b'Z')
                .chain(b'a'..=b'z')
                .chain(std::iter::once(b'_'))
                .collect();
            assert_eq!(
                start_transitions[0].symbol_onehot_encoding,
                Transition::convert_char_vec_to_symbol_onehot_encoding(char_vec)
            );
        }

        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"\D")?;

            let mut nfa = NFA::new();
            let nfa_result = nfa.add_ast_to_nfa(&parsed_ast, State(0), State(1));
            assert!(nfa_result.is_err());
            let nfa_error = nfa_result.err().unwrap();
            assert!(matches!(nfa_error, NegatedPerl));
        }

        Ok(())
    }

    #[test]
    fn test_concat_simple() -> Result<()> {
        let mut parser = RegexParser::new();
        let parsed_ast = parser.parse_into_ast(r"<\d>")?;

        let mut nfa = NFA::new();
        nfa.add_ast_to_nfa(&parsed_ast, State(0), State(1))?;

        let state_0_transitions = nfa.transitions.get(&State(0)).unwrap();
        assert_eq!(state_0_transitions.len(), 1);
        assert_eq!(state_0_transitions[0].from, State(0));
        assert_eq!(state_0_transitions[0].to, State(2));
        assert_eq!(
            state_0_transitions[0].symbol_onehot_encoding,
            Transition::convert_char_to_symbol_onehot_encoding('<')
        );

        let state_2_transitions = nfa.transitions.get(&State(2)).unwrap();
        assert_eq!(state_2_transitions.len(), 1);
        assert_eq!(state_2_transitions[0].from, State(2));
        assert_eq!(state_2_transitions[0].to, State(3));
        assert_eq!(
            state_2_transitions[0].symbol_onehot_encoding,
            DIGIT_TRANSITION
        );

        let state_3_transitions = nfa.transitions.get(&State(3)).unwrap();
        assert_eq!(state_3_transitions.len(), 1);
        assert_eq!(state_3_transitions[0].from, State(3));
        assert_eq!(state_3_transitions[0].to, State(1));
        assert_eq!(
            state_3_transitions[0].symbol_onehot_encoding,
            Transition::convert_char_to_symbol_onehot_encoding('>')
        );

        Ok(())
    }

    // #[test]
    // fn offset_test() {
    //     let mut nfa = NFA::new(State(0), State(1));
    //     nfa.add_state(State(0));
    //     nfa.add_state(State(1));
    //     nfa.add_transition(Transition {
    //         from: State(0),
    //         to: State(1),
    //         symbol_onehot_encoding: Transition::convert_char_to_symbol_onehot_encoding('a'),
    //         tag: -1,
    //     });
    //
    //     nfa.offset_states(2);
    //
    //     assert_eq!(nfa.start, State(2));
    //     assert_eq!(nfa.accept, State(3));
    //     assert_eq!(nfa.states.len(), 2);
    //     assert_eq!(nfa.transitions.len(), 1);
    //     assert_eq!(nfa.transitions.contains_key(&State(2)), true);
    //
    //     let transitions = nfa.transitions.get(&State(2)).unwrap();
    //     assert_eq!(transitions.len(), 1);
    //     assert_eq!(transitions[0].from, State(2));
    //     assert_eq!(transitions[0].to, State(3));
    // }
    //
    // #[test]
    // fn nfa_from_ast_literal() {
    //     let ast = AstNode::Literal(AstNodeLiteral::new('a'));
    //     let nfa = NFA::from_ast(&ast);
    //     assert_eq!(nfa.start, State(0));
    //     assert_eq!(nfa.accept, State(1));
    //
    //     let states = nfa.states;
    //     let transitions = nfa.transitions;
    //
    //     assert_eq!(states.len(), 2);
    //     assert_eq!(transitions.len(), 1);
    //     assert_eq!(transitions.contains_key(&State(0)), true);
    //
    //     let transitions_from_start = transitions.get(&State(0)).unwrap();
    //     assert_eq!(transitions_from_start.len(), 1);
    //     assert_eq!(transitions_from_start[0].from, State(0));
    //     assert_eq!(transitions_from_start[0].to, State(1));
    // }
    //
    // #[test]
    // fn nfa_from_ast_concat() {
    //     let ast = AstNode::Concat(AstNodeConcat::new(
    //         AstNode::Literal(AstNodeLiteral::new('a')),
    //         AstNode::Literal(AstNodeLiteral::new('b')),
    //     ));
    //     let nfa = NFA::from_ast(&ast);
    //     assert_eq!(nfa.states.len(), 4);
    //     assert_eq!(nfa.transitions.len(), 3);
    //     assert_eq!(nfa.start, State(0));
    //     assert_eq!(nfa.accept, State(3));
    //
    //     let transitions = nfa.transitions;
    //
    //     let transitions_from_start = transitions.get(&State(0)).unwrap();
    //     assert_eq!(transitions_from_start.len(), 1);
    //     assert_eq!(transitions_from_start[0].from, State(0));
    //     assert_eq!(transitions_from_start[0].to, State(1));
    //
    //     let transitions_from_1 = transitions.get(&State(1)).unwrap();
    //     assert_eq!(transitions_from_1.len(), 1);
    //     assert_eq!(transitions_from_1[0].from, State(1));
    //     assert_eq!(transitions_from_1[0].to, State(2));
    //
    //     let transitions_from_2 = transitions.get(&State(2)).unwrap();
    //     assert_eq!(transitions_from_2.len(), 1);
    //     assert_eq!(transitions_from_2[0].from, State(2));
    //     assert_eq!(transitions_from_2[0].to, State(3));
    //
    //     assert_eq!(transitions.contains_key(&State(3)), false);
    // }
    //
    // #[test]
    // fn nfa_from_ast_union() {
    //     let ast = AstNode::Union(AstNodeUnion::new(
    //         AstNode::Literal(AstNodeLiteral::new('a')),
    //         AstNode::Literal(AstNodeLiteral::new('b')),
    //     ));
    //     let nfa = NFA::from_ast(&ast);
    //     assert_eq!(nfa.states.len(), 6); // 6 states in total
    //     assert_eq!(nfa.transitions.len(), 5); // 5 nodes have transitions
    //
    //     assert_eq!(nfa.start, State(0));
    //     assert_eq!(nfa.accept, State(1));
    //
    //     let transitions = nfa.transitions;
    //
    //     let transitions_from_start = transitions.get(&State(0)).unwrap();
    //     assert_eq!(transitions_from_start.len(), 2);
    //     assert_eq!(transitions_from_start[0].from, State(0));
    //     assert_eq!(transitions_from_start[0].to, State(2));
    //     assert_eq!(transitions_from_start[1].from, State(0));
    //     assert_eq!(transitions_from_start[1].to, State(4));
    //
    //     let transitions_from_2 = transitions.get(&State(2)).unwrap();
    //     assert_eq!(transitions_from_2.len(), 1);
    //     assert_eq!(transitions_from_2[0].from, State(2));
    //     assert_eq!(transitions_from_2[0].to, State(3));
    //
    //     let transitions_from_4 = transitions.get(&State(4)).unwrap();
    //     assert_eq!(transitions_from_4.len(), 1);
    //     assert_eq!(transitions_from_4[0].from, State(4));
    //     assert_eq!(transitions_from_4[0].to, State(5));
    //
    //     let transitions_from_3 = transitions.get(&State(3)).unwrap();
    //     assert_eq!(transitions_from_3.len(), 1);
    //     assert_eq!(transitions_from_3[0].from, State(3));
    //     assert_eq!(transitions_from_3[0].to, State(1));
    //
    //     let transitions_from_5 = transitions.get(&State(5)).unwrap();
    //     assert_eq!(transitions_from_5.len(), 1);
    //     assert_eq!(transitions_from_5[0].from, State(5));
    //     assert_eq!(transitions_from_5[0].to, State(1));
    //
    //     assert_eq!(transitions.contains_key(&State(1)), false);
    // }
    //
    // #[test]
    // fn nfa_from_ast_star() {
    //     let ast = AstNode::Star(AstNodeStar::new(AstNode::Literal(AstNodeLiteral::new('a'))));
    //     let nfa = NFA::from_ast(&ast);
    //     assert_eq!(nfa.states.len(), 4);
    //     assert_eq!(nfa.transitions.len(), 3); // except the accept state, all other states have transitions
    //
    //     assert_eq!(nfa.start, State(0));
    //     assert_eq!(nfa.accept, State(3));
    //
    //     let transitions = nfa.transitions;
    //
    //     let transitions_from_start = transitions.get(&State(0)).unwrap();
    //     assert_eq!(transitions_from_start.len(), 2);
    //     assert_eq!(transitions_from_start[0].from, State(0));
    //     assert_eq!(transitions_from_start[0].to, State(1));
    //     assert_eq!(transitions_from_start[1].from, State(0));
    //     assert_eq!(transitions_from_start[1].to, State(3));
    //
    //     let transitions_from_1 = transitions.get(&State(1)).unwrap();
    //     assert_eq!(transitions_from_1.len(), 1);
    //     assert_eq!(transitions_from_1[0].from, State(1));
    //     assert_eq!(transitions_from_1[0].to, State(2));
    //
    //     let transitions_from_2 = transitions.get(&State(2)).unwrap();
    //     assert_eq!(transitions_from_2.len(), 2);
    //     assert_eq!(transitions_from_2[0].from, State(2));
    //     assert_eq!(transitions_from_2[0].to, State(1));
    //     assert_eq!(transitions_from_2[1].from, State(2));
    //     assert_eq!(transitions_from_2[1].to, State(3));
    // }
    //
    // #[test]
    // fn nfa_from_ast_plus() {
    //     let ast = AstNode::Plus(AstNodePlus::new(AstNode::Literal(AstNodeLiteral::new('a'))));
    //     let nfa = NFA::from_ast(&ast);
    //     assert_eq!(nfa.states.len(), 4);
    //     assert_eq!(nfa.transitions.len(), 3); // except the accept state, all other states have transitions
    //
    //     assert_eq!(nfa.start, State(0));
    //     assert_eq!(nfa.accept, State(3));
    //
    //     let transitions = nfa.transitions;
    //
    //     let transitions_from_start = transitions.get(&State(0)).unwrap();
    //     assert_eq!(transitions_from_start.len(), 1);
    //     assert_eq!(transitions_from_start[0].from, State(0));
    //     assert_eq!(transitions_from_start[0].to, State(1));
    //
    //     let transitions_from_1 = transitions.get(&State(1)).unwrap();
    //     assert_eq!(transitions_from_1.len(), 1);
    //     assert_eq!(transitions_from_1[0].from, State(1));
    //     assert_eq!(transitions_from_1[0].to, State(2));
    //
    //     let transitions_from_2 = transitions.get(&State(2)).unwrap();
    //     assert_eq!(transitions_from_2.len(), 2);
    //     assert_eq!(transitions_from_2[0].from, State(2));
    //     assert_eq!(transitions_from_2[0].to, State(1));
    //     assert_eq!(transitions_from_2[1].from, State(2));
    //     assert_eq!(transitions_from_2[1].to, State(3));
    // }
    //
    // #[test]
    // fn nfa_from_ast_optional() {
    //     let ast = AstNode::Optional(AstNodeOptional::new(AstNode::Literal(AstNodeLiteral::new(
    //         'a',
    //     ))));
    //     let nfa = NFA::from_ast(&ast);
    //     assert_eq!(nfa.states.len(), 4);
    //     assert_eq!(nfa.transitions.len(), 3); // except the accept state, all other states have transitions
    //
    //     assert_eq!(nfa.start, State(0));
    //     assert_eq!(nfa.accept, State(3));
    //
    //     let transitions = nfa.transitions;
    //
    //     let transitions_from_start = transitions.get(&State(0)).unwrap();
    //     assert_eq!(transitions_from_start.len(), 2);
    //     assert_eq!(transitions_from_start[0].from, State(0));
    //     assert_eq!(transitions_from_start[0].to, State(3));
    //     assert_eq!(transitions_from_start[1].from, State(0));
    //     assert_eq!(transitions_from_start[1].to, State(1));
    //
    //     let transitions_from_1 = transitions.get(&State(1)).unwrap();
    //     assert_eq!(transitions_from_1.len(), 1);
    //     assert_eq!(transitions_from_1[0].from, State(1));
    //     assert_eq!(transitions_from_1[0].to, State(2));
    //
    //     let transitions_from_2 = transitions.get(&State(2)).unwrap();
    //     assert_eq!(transitions_from_2.len(), 1);
    //     assert_eq!(transitions_from_2[0].from, State(2));
    //     assert_eq!(transitions_from_2[0].to, State(3));
    // }
    //
    // #[test]
    // fn nfa_simple_debug_print() {
    //     let ast = AstNode::Concat(AstNodeConcat::new(
    //         AstNode::Optional(AstNodeOptional::new(AstNode::Literal(AstNodeLiteral::new(
    //             'a',
    //         )))),
    //         AstNode::Literal(AstNodeLiteral::new('b')),
    //     ));
    //     let nfa = NFA::from_ast(&ast);
    //     println!("{:?}", nfa);
    // }
    //
    // #[test]
    // fn nfa_epsilon_closure() {
    //     let mut nfa = NFA::new(State(0), State(3));
    //     for i in 0..=10 {
    //         nfa.add_state(State(i));
    //     }
    //     nfa.add_epsilon_transition(State(0), State(1));
    //     nfa.add_epsilon_transition(State(1), State(2));
    //     nfa.add_epsilon_transition(State(0), State(2));
    //     nfa.add_transition(Transition {
    //         from: State(2),
    //         to: State(3),
    //         symbol_onehot_encoding: Transition::convert_char_to_symbol_onehot_encoding('a'),
    //         tag: -1,
    //     });
    //     nfa.add_epsilon_transition(State(3), State(5));
    //     nfa.add_epsilon_transition(State(3), State(4));
    //     nfa.add_epsilon_transition(State(4), State(5));
    //     nfa.add_epsilon_transition(State(5), State(3));
    //
    //     let closure = nfa.epsilon_closure(&vec![State(0)]);
    //     assert_eq!(closure.len(), 3);
    //     assert_eq!(closure.contains(&State(0)), true);
    //     assert_eq!(closure.contains(&State(1)), true);
    //     assert_eq!(closure.contains(&State(2)), true);
    //     assert_eq!(closure.contains(&State(3)), false);
    //     assert_eq!(closure.contains(&State(10)), false);
    //
    //     let closure = nfa.epsilon_closure(&vec![State(3)]);
    //     assert_eq!(closure.len(), 3);
    //     assert_eq!(closure.contains(&State(3)), true);
    //     assert_eq!(closure.contains(&State(4)), true);
    //     assert_eq!(closure.contains(&State(5)), true);
    // }
}
