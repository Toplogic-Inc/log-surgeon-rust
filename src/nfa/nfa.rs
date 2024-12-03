use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use crate::parser::ast_node::ast_node::AstNode;
use crate::parser::ast_node::ast_node_concat::AstNodeConcat;
use crate::parser::ast_node::ast_node_literal::AstNodeLiteral;
use crate::parser::ast_node::ast_node_optional::AstNodeOptional;
use crate::parser::ast_node::ast_node_plus::AstNodePlus;
use crate::parser::ast_node::ast_node_star::AstNodeStar;
use crate::parser::ast_node::ast_node_union::AstNodeUnion;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct State(pub usize);

pub struct Transition {
    from: State,
    to: State,
    symbol: Option<char>,
    tag: i16,
}

impl Debug for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:?} -> {:?}, symbol: {:?}",
            self.from, self.to, self.symbol
        )
    }
}

impl Transition {
    pub fn new(from: State, to: State, symbol: Option<char>, tag: i16) -> Self {
        Transition {
            from,
            to,
            symbol,
            tag,
        }
    }

    pub fn get_symbol(&self) -> Option<char> {
        self.symbol
    }

    pub fn get_to_state(&self) -> State {
        self.to.clone()
    }
}

pub(crate) struct NFA {
    start: State,
    accept: State,
    states: HashSet<State>,
    transitions: HashMap<State, Vec<Transition>>,
}

// NFA implementation for NFA construction from AST
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
                    tag: -1,
                });
                nfa
            }
            AstNode::Concat(ast_node) => {
                // create NFA for left hand side and this will be the result NFA
                let mut nfa = NFA::from_ast(&ast_node.get_op1());
                let offset = nfa.states.len();

                // create NFA for right hand side and offset the states by the number of states on the left hand side NFA
                let mut rhs_nfa = NFA::from_ast(&ast_node.get_op2());
                rhs_nfa.offset_states(offset);

                // add the states from the right hand side NFA to the result NFA
                nfa.states = nfa.states.union(&rhs_nfa.states).cloned().collect();

                // add the transitions from the right hand side NFA to the result NFA
                nfa.add_epsilon_transition(nfa.accept.clone(), rhs_nfa.start.clone());
                // the accept state of the right hand side NFA is the accept state of the result NFA,
                // the initial state of the result NFA is the initial state of the left hand side NFA, so no op
                nfa.accept = rhs_nfa.accept.clone();
                for (from, transitions) in rhs_nfa.transitions {
                    nfa.transitions
                        .entry(from)
                        .or_insert(vec![])
                        .extend(transitions);
                }

                nfa
            }
            AstNode::Union(ast_node) => {
                let start = State(0);
                let accept = State(1);
                let mut nfa = NFA::new(start.clone(), accept.clone());
                nfa.add_state(start.clone());
                nfa.add_state(accept.clone());
                let mut offset = 2;

                // Lambda function to handle NFA integration
                let mut integrate_nfa = |sub_nfa: &mut NFA| {
                    sub_nfa.offset_states(offset);
                    nfa.add_epsilon_transition(start.clone(), sub_nfa.start.clone());
                    nfa.add_epsilon_transition(sub_nfa.accept.clone(), accept.clone());
                    nfa.states = nfa.states.union(&sub_nfa.states).cloned().collect();
                    for (from, transitions) in sub_nfa.transitions.drain() {
                        nfa.transitions
                            .entry(from)
                            .or_insert(vec![])
                            .extend(transitions);
                    }
                    offset += sub_nfa.states.len();
                };

                let mut lhs_nfa = NFA::from_ast(&ast_node.get_op1());
                integrate_nfa(&mut lhs_nfa);

                let mut rhs_nfa = NFA::from_ast(&ast_node.get_op2());
                integrate_nfa(&mut rhs_nfa);

                nfa
            }
            AstNode::Star(ast_node) => {
                let mut sub_nfa = NFA::from_ast(ast_node.get_op1());
                sub_nfa.offset_states(1);

                let start = State(0);
                let accept = State(sub_nfa.states.len() + 1);

                let mut nfa = NFA::new(start.clone(), accept.clone());
                nfa.add_state(start.clone());
                nfa.add_state(accept.clone());

                // TODO: We may not need so many transitions
                nfa.add_epsilon_transition(start.clone(), sub_nfa.start.clone());
                nfa.add_epsilon_transition(start.clone(), accept.clone());
                nfa.add_epsilon_transition(sub_nfa.accept.clone(), sub_nfa.start.clone());
                nfa.add_epsilon_transition(sub_nfa.accept.clone(), accept.clone());

                nfa.states = nfa.states.union(&sub_nfa.states).cloned().collect();
                for (from, transitions) in sub_nfa.transitions {
                    nfa.transitions
                        .entry(from)
                        .or_insert(vec![])
                        .extend(transitions);
                }

                nfa
            }
            AstNode::Plus(ast_node) => {
                let mut sub_nfa = NFA::from_ast(ast_node.get_op1());
                sub_nfa.offset_states(1);

                let start = State(0);
                let accept = State(sub_nfa.states.len() + 1);

                let mut nfa = NFA::new(start.clone(), accept.clone());
                nfa.add_state(start.clone());
                nfa.add_state(accept.clone());

                // Very similar to the Star case, but we don't allow the empty string, so
                // we don't need the epsilon transition from start to accept
                nfa.add_epsilon_transition(start.clone(), sub_nfa.start.clone());
                nfa.add_epsilon_transition(sub_nfa.accept.clone(), sub_nfa.start.clone());
                nfa.add_epsilon_transition(sub_nfa.accept.clone(), accept.clone());

                nfa.states = nfa.states.union(&sub_nfa.states).cloned().collect();
                for (from, transitions) in sub_nfa.transitions {
                    nfa.transitions
                        .entry(from)
                        .or_insert(vec![])
                        .extend(transitions);
                }

                nfa
            }
            AstNode::Optional(ast_node) => {
                let mut sub_nfa = NFA::from_ast(ast_node.get_op1());
                sub_nfa.offset_states(1);

                let start = State(0);
                let accept = State(sub_nfa.states.len() + 1);

                let mut nfa = NFA::new(start.clone(), accept.clone());
                nfa.add_state(start.clone());
                nfa.add_state(accept.clone());

                // We can either have empty string (bypass)
                nfa.add_epsilon_transition(start.clone(), accept.clone());
                // Or we can have the string from the sub NFA
                nfa.add_epsilon_transition(start.clone(), sub_nfa.start.clone());
                nfa.add_epsilon_transition(sub_nfa.accept.clone(), accept.clone());

                nfa.states.extend(sub_nfa.states);
                for (from, transitions) in sub_nfa.transitions {
                    nfa.transitions
                        .entry(from)
                        .or_insert(vec![])
                        .extend(transitions);
                }

                nfa
            }
            AstNode::Group(ast_node) => NFA::from_ast(ast_node.get_op1()),
        }
    }

    pub fn new(start: State, accept: State) -> Self {
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
        self.transitions
            .entry(transition.from.clone())
            .or_insert(vec![])
            .push(transition);
    }

    fn add_epsilon_transition(&mut self, from: State, to: State) {
        self.add_transition(Transition {
            from,
            to,
            symbol: None,
            tag: -1,
        });
    }

    // Offset all states by a given amount
    fn offset_states(&mut self, offset: usize) {
        if offset == 0 {
            return;
        }

        // Update start and accept states
        self.start = State(self.start.0 + offset);
        self.accept = State(self.accept.0 + offset);

        // Update all states
        let mut new_states = HashSet::new();
        for state in self.states.iter() {
            new_states.insert(State(state.0 + offset));
        }
        self.states = new_states;

        // Update transitions in place by adding the offset to each state's "from" and "to" values
        let mut updated_transitions: HashMap<State, Vec<Transition>> = HashMap::new();
        for (start, transitions) in self.transitions.iter() {
            let updated_start = State(start.0 + offset);
            let updated_transitions_list: Vec<Transition> = transitions
                .iter()
                .map(|transition| Transition {
                    from: State(transition.from.0 + offset),
                    to: State(transition.to.0 + offset),
                    symbol: transition.symbol,
                    tag: transition.tag,
                })
                .collect();
            updated_transitions.insert(updated_start, updated_transitions_list);
        }

        self.transitions = updated_transitions;
    }
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
                if transition.symbol.is_none() {
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

// Test use only functions for DFA

#[cfg(test)]
impl NFA {
    pub fn test_extern_add_state(&mut self, state: State) {
        self.add_state(state);
    }

    pub fn test_extern_add_transition(&mut self, transition: Transition) {
        self.add_transition(transition);
    }

    pub fn test_extern_add_epsilon_transition(&mut self, from: State, to: State) {
        self.add_epsilon_transition(from, to);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_test() {
        let mut nfa = NFA::new(State(0), State(1));
        nfa.add_state(State(0));
        nfa.add_state(State(1));
        nfa.add_transition(Transition {
            from: State(0),
            to: State(1),
            symbol: Some('a'),
            tag: -1,
        });

        nfa.offset_states(2);

        assert_eq!(nfa.start, State(2));
        assert_eq!(nfa.accept, State(3));
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.transitions.len(), 1);
        assert_eq!(nfa.transitions.contains_key(&State(2)), true);

        let transitions = nfa.transitions.get(&State(2)).unwrap();
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].from, State(2));
        assert_eq!(transitions[0].to, State(3));
    }

    #[test]
    fn nfa_from_ast_literal() {
        let ast = AstNode::Literal(AstNodeLiteral::new('a'));
        let nfa = NFA::from_ast(&ast);
        assert_eq!(nfa.start, State(0));
        assert_eq!(nfa.accept, State(1));

        let states = nfa.states;
        let transitions = nfa.transitions;

        assert_eq!(states.len(), 2);
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions.contains_key(&State(0)), true);

        let transitions_from_start = transitions.get(&State(0)).unwrap();
        assert_eq!(transitions_from_start.len(), 1);
        assert_eq!(transitions_from_start[0].from, State(0));
        assert_eq!(transitions_from_start[0].to, State(1));
    }

    #[test]
    fn nfa_from_ast_concat() {
        let ast = AstNode::Concat(AstNodeConcat::new(
            AstNode::Literal(AstNodeLiteral::new('a')),
            AstNode::Literal(AstNodeLiteral::new('b')),
        ));
        let nfa = NFA::from_ast(&ast);
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.transitions.len(), 3);
        assert_eq!(nfa.start, State(0));
        assert_eq!(nfa.accept, State(3));

        let transitions = nfa.transitions;

        let transitions_from_start = transitions.get(&State(0)).unwrap();
        assert_eq!(transitions_from_start.len(), 1);
        assert_eq!(transitions_from_start[0].from, State(0));
        assert_eq!(transitions_from_start[0].to, State(1));

        let transitions_from_1 = transitions.get(&State(1)).unwrap();
        assert_eq!(transitions_from_1.len(), 1);
        assert_eq!(transitions_from_1[0].from, State(1));
        assert_eq!(transitions_from_1[0].to, State(2));

        let transitions_from_2 = transitions.get(&State(2)).unwrap();
        assert_eq!(transitions_from_2.len(), 1);
        assert_eq!(transitions_from_2[0].from, State(2));
        assert_eq!(transitions_from_2[0].to, State(3));

        assert_eq!(transitions.contains_key(&State(3)), false);
    }

    #[test]
    fn nfa_from_ast_union() {
        let ast = AstNode::Union(AstNodeUnion::new(
            AstNode::Literal(AstNodeLiteral::new('a')),
            AstNode::Literal(AstNodeLiteral::new('b')),
        ));
        let nfa = NFA::from_ast(&ast);
        assert_eq!(nfa.states.len(), 6); // 6 states in total
        assert_eq!(nfa.transitions.len(), 5); // 5 nodes have transitions

        assert_eq!(nfa.start, State(0));
        assert_eq!(nfa.accept, State(1));

        let transitions = nfa.transitions;

        let transitions_from_start = transitions.get(&State(0)).unwrap();
        assert_eq!(transitions_from_start.len(), 2);
        assert_eq!(transitions_from_start[0].from, State(0));
        assert_eq!(transitions_from_start[0].to, State(2));
        assert_eq!(transitions_from_start[1].from, State(0));
        assert_eq!(transitions_from_start[1].to, State(4));

        let transitions_from_2 = transitions.get(&State(2)).unwrap();
        assert_eq!(transitions_from_2.len(), 1);
        assert_eq!(transitions_from_2[0].from, State(2));
        assert_eq!(transitions_from_2[0].to, State(3));

        let transitions_from_4 = transitions.get(&State(4)).unwrap();
        assert_eq!(transitions_from_4.len(), 1);
        assert_eq!(transitions_from_4[0].from, State(4));
        assert_eq!(transitions_from_4[0].to, State(5));

        let transitions_from_3 = transitions.get(&State(3)).unwrap();
        assert_eq!(transitions_from_3.len(), 1);
        assert_eq!(transitions_from_3[0].from, State(3));
        assert_eq!(transitions_from_3[0].to, State(1));

        let transitions_from_5 = transitions.get(&State(5)).unwrap();
        assert_eq!(transitions_from_5.len(), 1);
        assert_eq!(transitions_from_5[0].from, State(5));
        assert_eq!(transitions_from_5[0].to, State(1));

        assert_eq!(transitions.contains_key(&State(1)), false);
    }

    #[test]
    fn nfa_from_ast_star() {
        let ast = AstNode::Star(AstNodeStar::new(AstNode::Literal(AstNodeLiteral::new('a'))));
        let nfa = NFA::from_ast(&ast);
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.transitions.len(), 3); // except the accept state, all other states have transitions

        assert_eq!(nfa.start, State(0));
        assert_eq!(nfa.accept, State(3));

        let transitions = nfa.transitions;

        let transitions_from_start = transitions.get(&State(0)).unwrap();
        assert_eq!(transitions_from_start.len(), 2);
        assert_eq!(transitions_from_start[0].from, State(0));
        assert_eq!(transitions_from_start[0].to, State(1));
        assert_eq!(transitions_from_start[1].from, State(0));
        assert_eq!(transitions_from_start[1].to, State(3));

        let transitions_from_1 = transitions.get(&State(1)).unwrap();
        assert_eq!(transitions_from_1.len(), 1);
        assert_eq!(transitions_from_1[0].from, State(1));
        assert_eq!(transitions_from_1[0].to, State(2));

        let transitions_from_2 = transitions.get(&State(2)).unwrap();
        assert_eq!(transitions_from_2.len(), 2);
        assert_eq!(transitions_from_2[0].from, State(2));
        assert_eq!(transitions_from_2[0].to, State(1));
        assert_eq!(transitions_from_2[1].from, State(2));
        assert_eq!(transitions_from_2[1].to, State(3));
    }

    #[test]
    fn nfa_from_ast_plus() {
        let ast = AstNode::Plus(AstNodePlus::new(AstNode::Literal(AstNodeLiteral::new('a'))));
        let nfa = NFA::from_ast(&ast);
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.transitions.len(), 3); // except the accept state, all other states have transitions

        assert_eq!(nfa.start, State(0));
        assert_eq!(nfa.accept, State(3));

        let transitions = nfa.transitions;

        let transitions_from_start = transitions.get(&State(0)).unwrap();
        assert_eq!(transitions_from_start.len(), 1);
        assert_eq!(transitions_from_start[0].from, State(0));
        assert_eq!(transitions_from_start[0].to, State(1));

        let transitions_from_1 = transitions.get(&State(1)).unwrap();
        assert_eq!(transitions_from_1.len(), 1);
        assert_eq!(transitions_from_1[0].from, State(1));
        assert_eq!(transitions_from_1[0].to, State(2));

        let transitions_from_2 = transitions.get(&State(2)).unwrap();
        assert_eq!(transitions_from_2.len(), 2);
        assert_eq!(transitions_from_2[0].from, State(2));
        assert_eq!(transitions_from_2[0].to, State(1));
        assert_eq!(transitions_from_2[1].from, State(2));
        assert_eq!(transitions_from_2[1].to, State(3));
    }

    #[test]
    fn nfa_from_ast_optional() {
        let ast = AstNode::Optional(AstNodeOptional::new(AstNode::Literal(AstNodeLiteral::new(
            'a',
        ))));
        let nfa = NFA::from_ast(&ast);
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.transitions.len(), 3); // except the accept state, all other states have transitions

        assert_eq!(nfa.start, State(0));
        assert_eq!(nfa.accept, State(3));

        let transitions = nfa.transitions;

        let transitions_from_start = transitions.get(&State(0)).unwrap();
        assert_eq!(transitions_from_start.len(), 2);
        assert_eq!(transitions_from_start[0].from, State(0));
        assert_eq!(transitions_from_start[0].to, State(3));
        assert_eq!(transitions_from_start[1].from, State(0));
        assert_eq!(transitions_from_start[1].to, State(1));

        let transitions_from_1 = transitions.get(&State(1)).unwrap();
        assert_eq!(transitions_from_1.len(), 1);
        assert_eq!(transitions_from_1[0].from, State(1));
        assert_eq!(transitions_from_1[0].to, State(2));

        let transitions_from_2 = transitions.get(&State(2)).unwrap();
        assert_eq!(transitions_from_2.len(), 1);
        assert_eq!(transitions_from_2[0].from, State(2));
        assert_eq!(transitions_from_2[0].to, State(3));
    }

    #[test]
    fn nfa_simple_debug_print() {
        let ast = AstNode::Concat(AstNodeConcat::new(
            AstNode::Optional(AstNodeOptional::new(AstNode::Literal(AstNodeLiteral::new(
                'a',
            )))),
            AstNode::Literal(AstNodeLiteral::new('b')),
        ));
        let nfa = NFA::from_ast(&ast);
        println!("{:?}", nfa);
    }

    #[test]
    fn nfa_epsilon_closure() {
        let mut nfa = NFA::new(State(0), State(3));
        for i in 0..=10 {
            nfa.add_state(State(i));
        }
        nfa.add_epsilon_transition(State(0), State(1));
        nfa.add_epsilon_transition(State(1), State(2));
        nfa.add_epsilon_transition(State(0), State(2));
        nfa.add_transition(Transition {
            from: State(2),
            to: State(3),
            symbol: Some('a'),
            tag: -1,
        });
        nfa.add_epsilon_transition(State(3), State(5));
        nfa.add_epsilon_transition(State(3), State(4));
        nfa.add_epsilon_transition(State(4), State(5));
        nfa.add_epsilon_transition(State(5), State(3));

        let closure = nfa.epsilon_closure(&vec![State(0)]);
        assert_eq!(closure.len(), 3);
        assert_eq!(closure.contains(&State(0)), true);
        assert_eq!(closure.contains(&State(1)), true);
        assert_eq!(closure.contains(&State(2)), true);
        assert_eq!(closure.contains(&State(3)), false);
        assert_eq!(closure.contains(&State(10)), false);

        let closure = nfa.epsilon_closure(&vec![State(3)]);
        assert_eq!(closure.len(), 3);
        assert_eq!(closure.contains(&State(3)), true);
        assert_eq!(closure.contains(&State(4)), true);
        assert_eq!(closure.contains(&State(5)), true);
    }
}
