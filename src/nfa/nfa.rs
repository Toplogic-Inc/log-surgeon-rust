use std::collections::{HashSet, HashMap};
use std::hash::Hash;

use crate::parser::ast_node::ast_node::AstNode;

#[derive(Clone, Debug)]
struct State(usize);

struct Transition {
    from: State,
    to: State,
    symbol: Option<char>,
}

struct NFA {
    start: State,
    accept: State,
    states: HashSet<State>,
    transitions: HashMap<State, Vec<Transition>>,
}

impl NFA {

    fn from_ast(ast: &AstNode) -> Self {
        match ast {
            AstNode::Literal(ast_node) => {
                let start = State(0);
                let accept = State(1);
                let mut nfa = NFA::new(start.clone(), accept.clone());
                nfa.add_state(start.clone());
                nfa.add_state(accept.clone());
                nfa.add_transition(Transition {
                    from: start.clone(),
                    to: accept.clone(),
                    symbol: Some(ast_node.get_value()),
                });
                nfa
            }
            AstNode::Concat(ast_node) => {


            }
            AstNode::Union(ast_node) => {


            }
            AstNode::Star(ast_node) => {


            }
            AstNode::Plus(ast_node) => {


            }
            AstNode::Optional(ast_node) => {


            }
            AstNode::Group(ast_node) => {

            }
        }
    }

    fn new(start: State, accept: State) -> Self {
        NFA {
            start,
            accept,
            states: HashSet::new(),
            transitions: HashMap::new(),
        }
    }

    fn add_state(&mut self, state: State) {
        self.states.insert(state);
    }

    fn add_transition(&mut self, transition: Transition) {
        self.transitions.entry(transition.from.clone()).or_insert(vec![]).push(transition);
    }

    fn add_epsilon_transition(&mut self, from: State, to: State) {
        self.add_transition(Transition {
            from,
            to,
            symbol: None,
        });
    }
}