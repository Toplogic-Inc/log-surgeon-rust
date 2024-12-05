use crate::nfa::nfa::NFA;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::process::id;

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
    dfa_to_accepted_nfa_state_mapping: Option<HashMap<State, Vec<(usize, crate::nfa::nfa::State)>>>, // to determine which NFA gets matched
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
            dfa_to_accepted_nfa_state_mapping: None,
        }
    }

    fn add_transition(
        &mut self,
        from_state: State,
        symbol: char,
        to_state: State,
        tag: Option<Tag>,
    ) {
        self.states.insert(from_state.clone());
        self.states.insert(to_state.clone());
        self.transitions
            .entry(from_state.clone())
            .or_insert_with(HashMap::new)
            .insert(
                symbol,
                Transition {
                    from_state,
                    symbol,
                    to_state,
                    tag,
                },
            );
    }

    fn simulate(&self, input: &str) -> (Option<HashSet<usize>>, bool) {
        let mut current_state = self.start.clone();

        // simulate the dfa
        for symbol in input.chars() {
            let transitions = self.transitions.get(&current_state);
            if transitions.is_none() {
                return (None, false);
            }
            let transitions = transitions.unwrap();
            let transition = transitions.get(&symbol);
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
                if let Some(dfa_to_accepted_nfa_state_mapping) =
                    &self.dfa_to_accepted_nfa_state_mapping
                {
                    let nfa_states: &Vec<(usize, crate::nfa::nfa::State)> =
                        dfa_to_accepted_nfa_state_mapping
                            .get(&current_state)
                            .unwrap();

                    let mut nfa_ids = HashSet::new();
                    for (nfa_id, state) in nfa_states.iter() {
                        nfa_ids.insert(*nfa_id);
                    }

                    return (Some(nfa_ids), true);
                }

                return (None, true);
            }
        }

        (None, false)
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

    fn combine_state_names(nfa_stats: &Vec<(usize, crate::nfa::nfa::State)>) -> String {
        let mut names = nfa_stats
            .iter()
            .map(|state| state.0.to_string() + "_" + &state.1 .0.to_string())
            .collect::<Vec<String>>();
        names.sort();

        names.join(",")
    }
}

impl DFA {
    fn from_nfa(nfa: NFA) -> DFA {
        let mut dfa_states: HashSet<State> = HashSet::new();
        let mut dfa_to_nfa_state_mapping: HashMap<State, Vec<crate::nfa::nfa::State>> =
            HashMap::new();
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
            let nfa_states: &Vec<crate::nfa::nfa::State> =
                dfa_to_nfa_state_mapping.get(&dfa_state.clone()).unwrap();

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
                let transitions: Option<&Vec<crate::nfa::nfa::Transition>> =
                    nfa.get_transitions_from_state(nfa_state);
                for transition in transitions.into_iter().flatten() {
                    let symbol = transition.get_symbol();

                    //We don't want to track epsilon transitions
                    if let Some(s) = symbol {
                        move_transitions_symbol_to_transitions_map
                            .entry(s)
                            .or_insert_with(Vec::new)
                            .push(transition);
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
                    dfa_to_nfa_state_mapping
                        .insert(State(destination_dfa_state.clone()), destination_nfa_states);
                    worklist.push(State(destination_dfa_state.clone()));
                }

                // Add the transition to the dfa
                dfa_transitions
                    .entry(dfa_state.clone())
                    .or_insert_with(HashMap::new)
                    .insert(
                        *symbol,
                        Transition {
                            from_state: dfa_state.clone(),
                            symbol: *symbol,
                            to_state: State(destination_dfa_state.clone()),
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
            dfa_to_accepted_nfa_state_mapping: None,
        }
    }

    fn from_multiple_nfas(nfas: Vec<NFA>) -> DFA {
        // All of the nodes now have a pair of identifiers,
        // 1. the NFA index within the list of NFAs
        // 2. the NFA state index within the NFA

        let mut dfa_states: HashSet<State> = HashSet::new();
        let mut dfa_to_nfa_state_mapping: HashMap<State, Vec<(usize, crate::nfa::nfa::State)>> =
            HashMap::new();
        let mut dfa_to_accepted_nfa_state_mapping: HashMap<
            State,
            Vec<(usize, crate::nfa::nfa::State)>,
        > = HashMap::new();
        let mut dfa_accept_states = HashSet::new();
        let mut dfa_transitions: HashMap<State, HashMap<char, Transition>> = HashMap::new();
        let mut worklist: Vec<State> = Vec::new();

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
        let start_epi_closure = DFA::epsilon_closure(&nfas, &nfa_starts);

        let start_state = DFA::combine_state_names(&start_epi_closure);
        dfa_states.insert(State(start_state.clone()));
        dfa_to_nfa_state_mapping.insert(State(start_state.clone()), start_epi_closure);
        worklist.push(State(start_state.clone()));

        // Process and add all dfa states
        while let Some(dfa_state) = worklist.pop() {
            let nfa_states: &Vec<(usize, crate::nfa::nfa::State)> =
                dfa_to_nfa_state_mapping.get(&dfa_state.clone()).unwrap();

            // Check if this dfa state is an accept state
            // Note: tIf any of the NFA states in this dfa state is an accept state, then this dfa state is an accept state
            for (idx, nfa_state) in nfa_states.iter() {
                if nfas.get(*idx).unwrap().get_accept() == *nfa_state {
                    dfa_to_accepted_nfa_state_mapping
                        .entry(dfa_state.clone())
                        .or_insert_with(Vec::new)
                        .push((*idx, nfa_state.clone()));
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
                    let symbol = transition.get_symbol();

                    //We don't want to track epsilon transitions
                    if let Some(s) = symbol {
                        move_transitions_symbol_to_transitions_map
                            .entry(s)
                            .or_insert_with(Vec::new)
                            .push((idx.clone(), transition));
                    }
                }
            }

            // Process the Epsilon Closure of the Move operation
            for (symbol, transitions) in move_transitions_symbol_to_transitions_map.iter() {
                // Collect all the destination NFA states
                let mut destination_nfa_states: Vec<(usize, crate::nfa::nfa::State)> = Vec::new();
                for (idx, transition) in transitions.iter() {
                    destination_nfa_states.push((*idx, (**transition).get_to_state()));
                }
                let destination_nfa_states = DFA::epsilon_closure(&nfas, &destination_nfa_states);

                // Check if the destination NFA states are already in the dfa states set
                let destination_dfa_state = DFA::combine_state_names(&destination_nfa_states);
                if !dfa_states.contains(&State(destination_dfa_state.clone())) {
                    println!("Inserting State {}", destination_dfa_state);
                    dfa_states.insert(State(destination_dfa_state.clone()));
                    dfa_to_nfa_state_mapping
                        .insert(State(destination_dfa_state.clone()), destination_nfa_states);
                    worklist.push(State(destination_dfa_state.clone()));
                }

                // Add the transition to the dfa
                dfa_transitions
                    .entry(dfa_state.clone())
                    .or_insert_with(HashMap::new)
                    .insert(
                        *symbol,
                        Transition {
                            from_state: dfa_state.clone(),
                            symbol: *symbol,
                            to_state: State(destination_dfa_state.clone()),
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
            dfa_to_accepted_nfa_state_mapping: Some(dfa_to_accepted_nfa_state_mapping),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dfa::dfa::Tag::Start;
    use crate::dfa::dfa::{State, DFA};
    use crate::nfa::nfa::NFA;
    use crate::{dfa, nfa};
    use std::collections::HashSet;

    #[test]
    fn test_dfa() {
        let start = dfa::dfa::State("0".parse().unwrap());
        let accept = dfa::dfa::State("1".parse().unwrap());
        let mut dfa = DFA::new(start.clone(), vec![accept.clone()]);
        dfa.add_transition(start.clone(), 'a', accept.clone(), None);
        dfa.add_transition(accept.clone(), 'b', start.clone(), None);

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
            Option::from('a'),
            -1,
        ));

        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(2),
            nfa::nfa::State(4),
            Option::from('a'),
            -1,
        ));

        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(3),
            nfa::nfa::State(5),
            Option::from('b'),
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
            Option::from('c'),
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
            Option::from('c'),
            -1,
        ));
        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(2),
            nfa::nfa::State(2),
            Option::from('c'),
            -1,
        ));
        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(2),
            nfa::nfa::State(3),
            Option::from('a'),
            -1,
        ));
        nfa.test_extern_add_transition(nfa::nfa::Transition::new(
            nfa::nfa::State(3),
            nfa::nfa::State(4),
            Option::from('b'),
            -1,
        ));

        nfa
    }

    #[test]
    fn test_nfa1_from_nfa_to_dfa() {
        let mut nfa = create_nfa1();
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
        assert_eq!(
            transitions_from_start_given_a.to_state,
            State("3,4,6".to_string())
        );

        let transitions_to_accept = dfa.transitions.get(&State("3,4,6".to_string())).unwrap();
        assert_eq!(transitions_to_accept.len(), 1);
        let transitions_to_accept_given_b = transitions_to_accept.get(&'b').unwrap();
        assert_eq!(
            transitions_to_accept_given_b.to_state,
            State("5,6".to_string())
        );

        // Check correctness given some examples
        assert_eq!(dfa.simulate("a"), (None, true));
        assert_eq!(dfa.simulate("ab"), (None, true));
        assert_eq!(dfa.simulate("aa"), (None, false));
        assert_eq!(dfa.simulate("abb"), (None, false));
        assert_eq!(dfa.simulate("aba"), (None, false));
    }

    #[test]
    fn test_nfa2_from_nfa_to_dfa() {
        let mut nfa = create_nfa2();
        let dfa = DFA::from_nfa(nfa);

        // Check correctness given some examples
        assert_eq!(dfa.simulate("c"), (None, true));
        assert_eq!(dfa.simulate("cc"), (None, true));
        assert_eq!(dfa.simulate("ccc"), (None, true));
        assert_eq!(dfa.simulate("cccc"), (None, true));
        assert_eq!(dfa.simulate("ccccab"), (None, false));
        assert_eq!(dfa.simulate("cab"), (None, false));
        assert_eq!(dfa.simulate(""), (None, true));
    }

    #[test]
    fn test_nfa3_from_nfa_to_dfa() {
        let mut nfa = create_nfa3();
        let dfa = DFA::from_nfa(nfa);

        // Check correctness given some examples
        assert_eq!(dfa.simulate("c"), (None, false));
        assert_eq!(dfa.simulate("cc"), (None, false));
        assert_eq!(dfa.simulate("ccc"), (None, false));
        assert_eq!(dfa.simulate("ccccc"), (None, false));
        assert_eq!(dfa.simulate("cccccab"), (None, true));
        assert_eq!(dfa.simulate("cab"), (None, true));
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

        assert_eq!(dfa.simulate("a"), (Some(HashSet::from([0])), true));
        assert_eq!(dfa.simulate("ab"), (Some(HashSet::from([0])), true));
        assert_eq!(dfa.simulate("aa"), (None, false));
        assert_eq!(dfa.simulate("abb"), (None, false));
        assert_eq!(dfa.simulate("aba"), (None, false));
        assert_eq!(dfa.simulate("c"), (Some(HashSet::from([1])), true));
        assert_eq!(dfa.simulate("cc"), (Some(HashSet::from([1])), true));
        assert_eq!(dfa.simulate("ccc"), (Some(HashSet::from([1])), true));
        assert_eq!(dfa.simulate("ccccc"), (Some(HashSet::from([1])), true));
        assert_eq!(dfa.simulate("cccccab"), (Some(HashSet::from([2])), true));
        assert_eq!(dfa.simulate("cab"), (Some(HashSet::from([2])), true));
        assert_eq!(dfa.simulate(""), (Some(HashSet::from([1])), true));
    }
}
