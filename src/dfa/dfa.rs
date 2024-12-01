use crate::nfa::nfa::NFA;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct State(String);

enum Tag {
    Start(usize),
    End(usize),
}

struct Transition {
    from_state: State,
    symbol: char,
    to_state: State,
    tag: Option<Tag>,
}

pub(crate) struct DFA {
    start: State,
    accept: Vec<State>,
    states: HashSet<State>,
    transitions: HashMap<State, HashMap<char, Transition>>, // from_state -> symbol -> to_state
}

impl DFA {
    fn new(start_state: State, accept_states: Vec<State>) -> Self {
        let mut _states = HashSet::new();
        _states.insert(start_state.clone());
        for state in accept_states.iter() {
            _states.insert(state.clone());
        }

        DFA {
            start: start_state,
            accept: accept_states,
            states: _states,
            transitions: HashMap::new(),
        }
    }

    fn add_transition(&mut self, from_state: State, symbol: char, to_state: State, tag: Option<Tag>) {
        self.states.insert(from_state.clone());
        self.states.insert(to_state.clone());
        self.transitions.entry(from_state.clone()).or_insert_with(HashMap::new).insert(symbol, Transition {
            from_state,
            symbol,
            to_state,
            tag,
        });
    }

    fn simulate(&self, input: &str) -> bool {
        let mut current_state = self.start.clone();

        // simulate the dfa
        for symbol in input.chars() {
            let transitions = self.transitions.get(&current_state);
            if transitions.is_none() {
                return false;
            }
            let transitions = transitions.unwrap();
            let transition = transitions.get(&symbol);
            if transition.is_none() {
                return false;
            }
            let next_state = Some(transition.unwrap().to_state.clone());
            if next_state.is_none() {
                return false;
            }
            current_state = next_state.unwrap();
        }

        // check if the current state is an accept state
        for accept_state in self.accept.iter() {
            if current_state == *accept_state {
                return true;
            }
        }

        false
    }
}

impl DFA {
    fn from_nfa(nfa: NFA) -> DFA{
        let mut dfa_states: HashSet<State> = HashSet::new();
        let mut dfa_to_nfa_state_mapping: HashMap<State, Vec<crate::nfa::nfa::State>> = HashMap::new();
        let mut dfa_accept_states = HashSet::new();
        let mut dfa_transitions: HashMap<State, HashMap<char, Transition>> = HashMap::new();
        let mut worklist: Vec<State> = Vec::new();

        // Start with the epsilon closure of the start state
        let nfa_start = nfa.get_start();
        let start_epi_closure = nfa.epsilon_closure(&vec![nfa_start]);
        let start_state = NFA::get_combined_state_names(&start_epi_closure);
        dfa_states.insert(State(start_state.clone()));
        dfa_to_nfa_state_mapping.insert(State(start_state.clone()), start_epi_closure);
        worklist.push(State(start_state.clone()));

        // Process and add all dfa states
        while let Some(dfa_state) = worklist.pop() {
            let nfa_states: &Vec<crate::nfa::nfa::State> = dfa_to_nfa_state_mapping.get(&dfa_state.clone()).unwrap();

            // Check if this dfa state is an accept state
            // Note: tIf any of the NFA states in this dfa state is an accept state, then this dfa state is an accept state
            for nfa_state in nfa_states.iter() {
                if nfa.get_accept() == *nfa_state {
                    dfa_accept_states.insert(dfa_state.clone());
                }
            }

            // Process the Move operation for all transitions in the NFA states set
            // The map stores all the transitions given a symbol for all the NFA states in the current dfa state
            let mut move_transitions_symbol_to_transitions_map = HashMap::new();
            for nfa_state in nfa_states.iter() {
                let transitions: Option<&Vec<crate::nfa::nfa::Transition>> = nfa.get_transitions_from_state(nfa_state);
                for transition in transitions.into_iter().flatten() {
                    let symbol = transition.get_symbol();

                    //We don't want to track epsilon transitions
                    if let Some(s) = symbol {
                        move_transitions_symbol_to_transitions_map.entry(s).or_insert_with(Vec::new).push(transition);
                    }
                }
            }

            // Process the Epsilon Closure of the Move operation
            for (symbol, transitions) in move_transitions_symbol_to_transitions_map.iter() {
                // Collect all the destination NFA states
                let mut destination_nfa_states = Vec::new();
                for transition in transitions.iter() {
                    destination_nfa_states.push((**transition).get_to_state());
                }
                let destination_nfa_states = nfa.epsilon_closure(&destination_nfa_states);

                // Check if the destination NFA states are already in the dfa states set
                let destination_dfa_state = NFA::get_combined_state_names(&destination_nfa_states);
                if !dfa_states.contains(&State(destination_dfa_state.clone())) {
                    println!("Inserting State {}", destination_dfa_state);
                    dfa_states.insert(State(destination_dfa_state.clone()));
                    dfa_to_nfa_state_mapping.insert(State(destination_dfa_state.clone()), destination_nfa_states);
                    worklist.push(State(destination_dfa_state.clone()));
                }

                // Add the transition to the dfa
                dfa_transitions.entry(dfa_state.clone()).or_insert_with(HashMap::new).insert(*symbol, Transition {
                    from_state: dfa_state.clone(),
                    symbol: *symbol,
                    to_state: State(destination_dfa_state.clone()),
                    tag: None,
                });
            }

        }

        DFA {
            start: State(start_state),
            accept: dfa_accept_states.into_iter().collect(),
            states: dfa_states,
            transitions: dfa_transitions,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{dfa, nfa};
    use crate::dfa::dfa::{State, DFA};
    use crate::dfa::dfa::Tag::Start;
    use crate::nfa::nfa::NFA;

    #[test]
    fn test_dfa() {
        let start = dfa::dfa::State("0".parse().unwrap());
        let accept = dfa::dfa::State("1".parse().unwrap());
        let mut dfa = DFA::new(start.clone(), vec![accept.clone()]);
        dfa.add_transition(start.clone(), 'a', accept.clone(), None);
        dfa.add_transition(accept.clone(), 'b', start.clone(), None);

        assert_eq!(dfa.simulate("ab"), false);
        assert_eq!(dfa.simulate("a"), true);
        assert_eq!(dfa.simulate("b"), false);
        assert_eq!(dfa.simulate("ba"), false);
    }

    #[test]
    fn test_easy_from_nfa_to_dfa() {
        // input NFA
        // 0 -> 1 epsilon
        // 0 -> 2 epsilon
        // 1 -> 3 a
        // 2 -> 4 a
        // 3 -> 5 b
        // 4 -> 6 epsilon
        // 5 -> 6 epsilon
        // 0: start state
        // 6: accept state
        // Should only match "a" or "ab"

        let mut nfa = NFA::new(nfa::nfa::State(0), nfa::nfa::State(6));

        for i in 1..=6 {
            nfa.test_extern_add_state(nfa::nfa::State(i));
        }

        nfa.test_extern_add_epsilon_transition(nfa::nfa::State(0), nfa::nfa::State(1));
        nfa.test_extern_add_epsilon_transition(nfa::nfa::State(0), nfa::nfa::State(2));

        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(1),
            nfa::nfa::State(3),
            Option::from('a'),
            -1
        ));

        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(2),
            nfa::nfa::State(4),
            Option::from('a'),
            -1
        ));

        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(3),
            nfa::nfa::State(5),
            Option::from('b'),
            -1
        ));

        nfa.test_extern_add_epsilon_transition(nfa::nfa::State(5), nfa::nfa::State(6));
        nfa.test_extern_add_epsilon_transition(nfa::nfa::State(4), nfa::nfa::State(6));

        let dfa = DFA::from_nfa(nfa);

        assert_eq!(dfa.start, dfa::dfa::State("0,1,2".to_string()));
        assert_eq!(dfa.accept.len(), 2);
        assert_eq!(dfa.accept.contains(&State("3,4,6".to_string())), true);
        assert_eq!(dfa.accept.contains(&State("5,6".to_string())), true);

        assert_eq!(dfa.states.len(), 3);
        assert_eq!(dfa.states.contains(&State("0,1,2".to_string())), true);
        assert_eq!(dfa.states.contains(&State("3,4,6".to_string())), true);
        assert_eq!(dfa.states.contains(&State("5,6".to_string())), true);

        assert_eq!(dfa.transitions.len(), 2);
        let transitions_from_start = dfa.transitions.get(&State("0,1,2".to_string())).unwrap();
        assert_eq!(transitions_from_start.len(), 1);
        let transitions_from_start_given_a = transitions_from_start.get(&'a').unwrap();
        assert_eq!(transitions_from_start_given_a.to_state, State("3,4,6".to_string()));

        let transitions_to_accept = dfa.transitions.get(&State("3,4,6".to_string())).unwrap();
        assert_eq!(transitions_to_accept.len(), 1);
        let transitions_to_accept_given_b = transitions_to_accept.get(&'b').unwrap();
        assert_eq!(transitions_to_accept_given_b.to_state, State("5,6".to_string()));

        // Check correctness given some examples
        assert_eq!(dfa.simulate("a"), true);
        assert_eq!(dfa.simulate("ab"), true);
        assert_eq!(dfa.simulate("aa"), false);
        assert_eq!(dfa.simulate("abb"), false);
        assert_eq!(dfa.simulate("aba"), false);
    }
}