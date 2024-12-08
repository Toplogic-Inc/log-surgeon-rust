use crate::nfa::nfa::NFA;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct State(usize);

enum Tag {
    Start(usize),
    End(usize),
}

struct Transition {
    from_state: State,
    symbol_onehot_encoding: u128,
    to_state: State,
    tag: Option<Tag>,
}

pub(crate) struct DFA {
    start: State,
    accept: Vec<State>,
    states: Vec<State>,
    transitions: Vec<HashMap<u128, Transition>>, // from_state -> symbol -> to_state
    dfa_to_accepted_nfa_state_mapping: Vec<Option<(usize, crate::nfa::nfa::State)>>, // to determine which NFA gets matched
}

impl DFA {
    // Cretae a new DFA with only the start state: 0
    fn new() -> Self {
        let mut _states = Vec::new();
        _states.push(State(0)); // start state is always 0

        let mut _transitions = Vec::new();
        _transitions.push(HashMap::new());

        DFA {
            start: State(0),
            accept: Vec::new(),
            states: _states,
            transitions: _transitions,
            dfa_to_accepted_nfa_state_mapping: Vec::new(),
        }
    }

    fn add_transition(
        &mut self,
        from_state: State,
        symbol_onehot_encoding: u128,
        to_state: State,
        tag: Option<Tag>,
    ) {
        assert!(self.states.len() > from_state.0);
        assert!(self.transitions.len() > from_state.0);
        assert!(self.states.len() > to_state.0);

        self.transitions
            .get_mut(from_state.0)
            .unwrap()
            .insert(
                symbol_onehot_encoding,
                Transition {
                    from_state,
                    symbol_onehot_encoding,
                    to_state,
                    tag,
                },
            );
    }

    fn get_transition(
        transitions_map: &HashMap<u128, Transition>,
        symbol: char,
    ) -> Option<&Transition> {
        for (transition_symbol, transition) in transitions_map.iter() {
            if (*transition_symbol & (1 << (symbol as u8))) != 0 {
                return Some(transition);
            }
        }

        None
    }

    fn simulate(&self, input: &str) -> (Option<usize>, bool) {
        let mut current_state = self.start.clone();

        // simulate the dfa
        for symbol in input.chars() {
            let transitions = self.transitions.get(current_state.0);
            if transitions.is_none() {
                return (None, false);
            }
            let transitions = transitions.unwrap();
            let transition = DFA::get_transition(transitions, symbol);
            if transition.is_none() {
                return (None, false);
            }
            let next_state = Some(transition.unwrap().to_state.clone());
            if next_state.is_none() {
                return (None, false);
            }
            current_state = next_state.unwrap();
        }

        // check if the current state is an accept state
        for accept_state in self.accept.iter() {
            if current_state == *accept_state {
                let nfa_state =
                    self.dfa_to_accepted_nfa_state_mapping
                        .get(current_state.0);

                if nfa_state.is_none() {
                    println!("[WARN] This should only happen when the DFA is created from scratch, not created from NFA(s)");
                    return (None, true)
                }

                let nfa_state =
                    self.dfa_to_accepted_nfa_state_mapping
                        .get(current_state.0).unwrap();

                assert_eq!(nfa_state.is_some(), true);
                return (Some(nfa_state.clone().unwrap().0), true)
            }
        }

        (None, false)
    }

    fn reset_simulation(&self) {
        // TODO: Implement this function
    }

    fn simulate_single_char(&self, input: char) -> Option<usize> {
        // TODO: Implement this function
        None
    }
}

// Helper functions for converting multiple NFAs to a single DFA
impl DFA {
    fn epsilon_closure(
        nfas: &Vec<NFA>,
        states: &Vec<(usize, crate::nfa::nfa::State)>,
    ) -> Vec<(usize, crate::nfa::nfa::State)> {
        let mut closure = Vec::new();

        for (idx, nfa_start) in states.iter() {
            let single_nfa_start_epi_closure: Vec<crate::nfa::nfa::State> = nfas
                .get(*idx)
                .unwrap()
                .epsilon_closure(&vec![nfa_start.clone()]);
            for state in single_nfa_start_epi_closure.iter() {
                closure.push((*idx, state.clone()));
            }
        }

        closure
    }
}

impl DFA {
    fn from_multiple_nfas(nfas: Vec<NFA>) -> DFA {
        // All of the nodes now have a pair of identifiers,
        // 1. the NFA index within the list of NFAs
        // 2. the NFA state index within the NFA

        // variables to create a new DFA
        let mut dfa_states: Vec<State> = Vec::new();
        let mut dfa_to_nfa_state_mapping: Vec<Rc<Vec<(usize, crate::nfa::nfa::State)>>> =
            Vec::new();
        let mut dfa_to_accepted_nfa_state_mapping: Vec<Option<(usize, crate::nfa::nfa::State)>> = Vec::new();
        let mut dfa_accept_states = HashSet::new();
        let mut dfa_transitions: Vec<HashMap<u128, Transition>> = Vec::new();

        // local variables to help create the DFA
        let mut l_worklist: Vec<State> = Vec::new();
        let mut l_nfa_states_to_dfa_mapping: HashMap<
            Rc<Vec<(usize, crate::nfa::nfa::State)>>,
            State,
        > = HashMap::new();

        // Start with the epsilon closure of the start state
        let mut nfa_starts = Vec::new();
        for (idx, nfa) in nfas.iter().enumerate() {
            nfa_starts.push((idx, nfa.get_start()));
        }

        // let mut start_epi_closure: Vec<(usize, crate::nfa::nfa::State)> = vec![];
        // for (idx, nfa_start) in nfa_starts.iter() {
        //     let single_nfa_start_epi_closure : crate::nfa::nfa::State = nfas.get(idx).epsilon_closure(&vec![nfa_start]);
        //     start_epi_closure.push((idx, single_nfa_start_epi_closure));
        // }
        let start_epi_closure: Rc<Vec<(usize, crate::nfa::nfa::State)>> =
            Rc::new(DFA::epsilon_closure(&nfas, &nfa_starts));

        let start_state = 0usize;
        dfa_states.push(State(start_state));
        dfa_transitions.push(HashMap::new());
        dfa_to_nfa_state_mapping.push(start_epi_closure.clone());
        dfa_to_accepted_nfa_state_mapping.push(None);
        l_nfa_states_to_dfa_mapping.insert(start_epi_closure, State(start_state));
        l_worklist.push(State(start_state));

        // Process and add all dfa states
        while let Some(dfa_state) = l_worklist.pop() {
            let nfa_states: &Vec<(usize, crate::nfa::nfa::State)> =
                dfa_to_nfa_state_mapping.get(dfa_state.0).unwrap();

            // Check if this dfa state is an accept state
            // Note: If any of the NFA states in this dfa state is an accept state, then this dfa state is an accept state
            for (idx, nfa_state) in nfa_states.iter() {
                if nfas.get(*idx).unwrap().get_accept() == *nfa_state {
                    dfa_to_accepted_nfa_state_mapping.get_mut(dfa_state.0).as_mut().unwrap().replace((*idx, nfa_state.clone()));
                    dfa_accept_states.insert(dfa_state.clone());
                }
            }

            // Process the Move operation for all transitions in the NFA states set
            // The map stores all the transitions given a symbol for all the NFA states in the current dfa state
            let mut move_transitions_symbol_to_transitions_map = HashMap::new();
            for (idx, nfa_state) in nfa_states.iter() {
                let transitions: Option<&Vec<crate::nfa::nfa::Transition>> = nfas
                    .get(*idx)
                    .unwrap()
                    .get_transitions_from_state(nfa_state);
                for transition in transitions.into_iter().flatten() {
                    let symbol_onehot_encoding = transition.get_symbol_onehot_encoding();

                    //We don't want to track epsilon transitions
                    if symbol_onehot_encoding != 0 {
                        move_transitions_symbol_to_transitions_map
                            .entry(symbol_onehot_encoding)
                            .or_insert_with(Vec::new)
                            .push((idx.clone(), transition));
                    }
                }
            }

            // Process the Epsilon Closure of the Move operation
            for (symbol_onehot_encoding, transitions) in
                move_transitions_symbol_to_transitions_map.iter()
            {
                // Collect all the destination NFA states
                let mut destination_nfa_states: Vec<(usize, crate::nfa::nfa::State)> = Vec::new();
                for (idx, transition) in transitions.iter() {
                    destination_nfa_states.push((*idx, (**transition).get_to_state()));
                }
                let destination_nfa_states =
                    Rc::new(DFA::epsilon_closure(&nfas, &destination_nfa_states));

                // Check if the destination NFA states are already in the dfa states set
                // let destination_dfa_state = DFA::combine_state_names(&destination_nfa_states);
                if !l_nfa_states_to_dfa_mapping.contains_key(&destination_nfa_states) {
                    // We need to add a new state to the DFA
                    let destination_dfa_state_idx = dfa_states.len();
                    println!("Inserting State {}", destination_dfa_state_idx);

                    dfa_states.push(State(destination_dfa_state_idx));
                    dfa_transitions.push(HashMap::new());
                    dfa_to_accepted_nfa_state_mapping.push(None);
                    dfa_to_nfa_state_mapping.push(destination_nfa_states.clone());
                    l_nfa_states_to_dfa_mapping.insert(
                        destination_nfa_states.clone(),
                        State(destination_dfa_state_idx),
                    );
                    l_worklist.push(State(destination_dfa_state_idx));
                }
                let destination_dfa_state = l_nfa_states_to_dfa_mapping
                    .get(&destination_nfa_states)
                    .unwrap();

                // Add the transition to the dfa
                dfa_transitions
                    .get_mut(dfa_state.0)
                    .unwrap()
                    .insert(
                        *symbol_onehot_encoding,
                        Transition {
                            from_state: dfa_state.clone(),
                            symbol_onehot_encoding: *symbol_onehot_encoding,
                            to_state: destination_dfa_state.clone(),
                            tag: None,
                        },
                    );
            }
        }

        DFA {
            start: State(start_state),
            accept: dfa_accept_states.into_iter().collect(),
            states: dfa_states,
            transitions: dfa_transitions,
            dfa_to_accepted_nfa_state_mapping,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dfa::dfa::{State, DFA};
    use crate::nfa::nfa::NFA;
    use crate::{dfa, nfa};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_dfa() {
        let start = dfa::dfa::State(0);
        let accept = dfa::dfa::State(1);
        let mut dfa = DFA::new();

        dfa.states.push(accept.clone());
        dfa.transitions.push(HashMap::new());
        dfa.accept.push(accept.clone());

        dfa.add_transition(
            start.clone(),
            nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('a'),
            accept.clone(),
            None,
        );
        dfa.add_transition(
            accept.clone(),
            nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('b'),
            start.clone(),
            None,
        );

        assert_eq!(dfa.simulate("ab"), (None, false));
        assert_eq!(dfa.simulate("a"), (None, true));
        assert_eq!(dfa.simulate("b"), (None, false));
        assert_eq!(dfa.simulate("ba"), (None, false));
    }

    #[cfg(test)]
    fn create_nfa1() -> NFA {
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
            nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('a'),
            -1,
        ));

        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(2),
            nfa::nfa::State(4),
            nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('a'),
            -1,
        ));

        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(3),
            nfa::nfa::State(5),
            nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('b'),
            -1,
        ));

        nfa.test_extern_add_epsilon_transition(nfa::nfa::State(5), nfa::nfa::State(6));
        nfa.test_extern_add_epsilon_transition(nfa::nfa::State(4), nfa::nfa::State(6));

        nfa
    }

    #[cfg(test)]
    fn create_nfa2() -> NFA {
        // input NFA
        // 0 -> 1 epsilon
        // 1 -> 1 c
        // 1 -> 2 epsilon
        // Should match "c*"

        let mut nfa = NFA::new(nfa::nfa::State(0), nfa::nfa::State(2));
        nfa.test_extern_add_state(nfa::nfa::State(9));
        nfa.test_extern_add_state(nfa::nfa::State(1));
        nfa.test_extern_add_state(nfa::nfa::State(2));

        nfa.test_extern_add_epsilon_transition(nfa::nfa::State(0), nfa::nfa::State(1));
        nfa.test_extern_add_epsilon_transition(nfa::nfa::State(1), nfa::nfa::State(2));
        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(1),
            nfa::nfa::State(1),
            nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('c'),
            -1,
        ));

        nfa
    }

    #[cfg(test)]
    fn create_nfa3() -> NFA {
        // input NFA
        // 0 -> 1 epsilon
        // 1 -> 2 c
        // 2 -> 2 c
        // 2 -> 3 a
        // 3 -> 4 b
        // 4 -> 5 epsilon
        // Should match "c+ab"

        let mut nfa = NFA::new(nfa::nfa::State(0), nfa::nfa::State(5));
        for i in 1..=5 {
            nfa.test_extern_add_state(nfa::nfa::State(i));
        }

        nfa.test_extern_add_epsilon_transition(nfa::nfa::State(0), nfa::nfa::State(1));
        nfa.test_extern_add_epsilon_transition(nfa::nfa::State(4), nfa::nfa::State(5));

        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(1),
            nfa::nfa::State(2),
            nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('c'),
            -1,
        ));
        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(2),
            nfa::nfa::State(2),
            nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('c'),
            -1,
        ));
        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(2),
            nfa::nfa::State(3),
            nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('a'),
            -1,
        ));
        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(3),
            nfa::nfa::State(4),
            nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('b'),
            -1,
        ));

        nfa
    }

    #[test]
    fn test_nfa1_from_nfa_to_dfa() {
        let nfa = create_nfa1();
        let dfa = DFA::from_multiple_nfas(vec![nfa]);

        // 0 1 2 : 0
        // 3 4 6 : 1
        // 5 6 : 2

        assert_eq!(dfa.start, dfa::dfa::State(0));
        assert_eq!(dfa.accept.len(), 2);
        assert_eq!(dfa.accept.contains(&State(1)), true);
        assert_eq!(dfa.accept.contains(&State(2)), true);
        //
        assert_eq!(dfa.states.len(), 3);
        assert_eq!(dfa.states.contains(&State(0)), true);
        assert_eq!(dfa.states.contains(&State(1)), true);
        assert_eq!(dfa.states.contains(&State(2)), true);
        //
        assert_eq!(dfa.transitions.len(), 3);
        let transitions_from_start = dfa.transitions.get(0).unwrap();
        assert_eq!(transitions_from_start.len(), 1);
        let transitions_from_start_given_a = transitions_from_start
            .get(&nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('a'))
            .unwrap();
        assert_eq!(transitions_from_start_given_a.to_state, State(1));

        let transitions_to_accept = dfa.transitions.get(1).unwrap();
        assert_eq!(transitions_to_accept.len(), 1);
        let transitions_to_accept_given_b = transitions_to_accept
            .get(&nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding('b'))
            .unwrap();
        assert_eq!(transitions_to_accept_given_b.to_state, State(2));

        // Check correctness given some examples
        assert_eq!(dfa.simulate("a"), (Some(0usize), true));
        assert_eq!(dfa.simulate("ab"), (Some(0usize), true));
        assert_eq!(dfa.simulate("aa"), (None, false));
        assert_eq!(dfa.simulate("abb"), (None, false));
        assert_eq!(dfa.simulate("aba"), (None, false));
    }

    #[test]
    fn test_nfa2_from_nfa_to_dfa() {
        let nfa = create_nfa2();
        let dfa = DFA::from_multiple_nfas(vec![nfa]);

        // Check correctness given some examples
        assert_eq!(dfa.simulate("c"), (Some(0usize), true));
        assert_eq!(dfa.simulate("cc"), (Some(0usize), true));
        assert_eq!(dfa.simulate("ccc"), (Some(0usize), true));
        assert_eq!(dfa.simulate("cccc"), (Some(0usize), true));
        assert_eq!(dfa.simulate("ccccab"), (None, false));
        assert_eq!(dfa.simulate("cab"), (None, false));
        assert_eq!(dfa.simulate(""), (Some(0usize), true));
    }

    #[test]
    fn test_nfa3_from_nfa_to_dfa() {
        let nfa = create_nfa3();
        let dfa = DFA::from_multiple_nfas(vec![nfa]);

        // Check correctness given some examples
        assert_eq!(dfa.simulate("c"), (None, false));
        assert_eq!(dfa.simulate("cc"), (None, false));
        assert_eq!(dfa.simulate("ccc"), (None, false));
        assert_eq!(dfa.simulate("ccccc"), (None, false));
        assert_eq!(dfa.simulate("cccccab"), (Some(0usize), true));
        assert_eq!(dfa.simulate("cab"), (Some(0usize), true));
        assert_eq!(dfa.simulate("ab"), (None, false));
        assert_eq!(dfa.simulate(""), (None, false));
    }

    #[test]
    fn test_easy_from_multi_nfas_to_dfa() {
        let nfa1 = create_nfa1();
        let nfa2 = create_nfa2();
        let nfa3 = create_nfa3();

        let dfa = DFA::from_multiple_nfas(vec![nfa1, nfa2, nfa3]);

        // Check correctness given some examples
        // Should match:
        // "a" or "ab"
        // "c*"
        // "c+ab"

        assert_eq!(dfa.simulate("a"), (Some(0usize), true));
        assert_eq!(dfa.simulate("ab"), (Some(0usize), true));
        assert_eq!(dfa.simulate("aa"), (None, false));
        assert_eq!(dfa.simulate("abb"), (None, false));
        assert_eq!(dfa.simulate("aba"), (None, false));
        assert_eq!(dfa.simulate("c"), (Some(1usize), true));
        assert_eq!(dfa.simulate("cc"), (Some(1usize), true));
        assert_eq!(dfa.simulate("ccc"), (Some(1usize), true));
        assert_eq!(dfa.simulate("ccccc"), (Some(1usize), true));
        assert_eq!(dfa.simulate("cccccab"), (Some(2usize), true));
        assert_eq!(dfa.simulate("cab"), (Some(2usize), true));
        assert_eq!(dfa.simulate(""), (Some(1usize), true));
    }
}
