use crate::nfa::nfa::NFA;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State(usize);

#[derive(Clone)]
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

impl Debug for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if 0 == self.symbol_onehot_encoding {
            return write!(
                f,
                "{:?} -> {:?}, symbol: {}",
                self.from_state, self.to_state, "epsilon"
            );
        }

        let mut char_vec: Vec<char> = Vec::new();
        for i in 0..128u8 {
            let mask = 1u128 << i;
            if mask & self.symbol_onehot_encoding == mask {
                char_vec.push(i as char);
            }
        }
        write!(
            f,
            "{:?} -> {:?}, symbol: {:?}",
            self.from_state, self.to_state, char_vec
        )
    }
}

pub(crate) struct DFA {
    start: State,
    accept: Vec<State>,
    states: Vec<State>,
    transitions: Vec<Vec<Option<Transition>>>, // from_state -> symbol[index in the length 128 vector] -> transition
    dfa_to_accepted_nfa_state_mapping: Vec<Option<(usize, crate::nfa::nfa::State)>>, // to determine which NFA gets matched
}

impl Debug for DFA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DFA( start: {:?}, accept: {:?}, states: {:?}, transitions: {{\n",
            self.start, self.accept, self.states
        )?;

        for state in &self.states {
            let state_idx = state.0;
            if self.transitions[state_idx].is_empty() {
                continue;
            }
            write!(f, "\t{:?}:\n", state)?;
            for transition_option in self.transitions[state_idx].iter() {
                if transition_option.is_none() {
                    continue;
                }
                write!(f, "\t\t{:?}\n", transition_option.as_ref().unwrap())?;
            }
        }

        write!(f, "}} )")
    }
}

pub(crate) struct DfaSimulator {
    dfa: Rc<DFA>,
    current_state: State,
}

impl DFA {
    // Cretae a new DFA with only the start state: 0
    fn new() -> Self {
        let mut _states = Vec::new();
        _states.push(State(0)); // start state is always 0

        let mut _transitions = Vec::new();
        let mut vector = Vec::with_capacity(128);
        for _ in 0..128 {
            vector.push(None::<Transition>);
        }
        _transitions.push(vector);

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

        for i in 0..128 {
            if (symbol_onehot_encoding & (1 << i)) != 0 {
                assert_eq!(self.transitions[from_state.0].len(), 128);
                self.transitions[from_state.0][i] = Some(Transition {
                    from_state: from_state.clone(),
                    symbol_onehot_encoding,
                    to_state: to_state.clone(),
                    tag: tag.clone(),
                });
            }
        }
    }

    fn get_transition(
        transitions_map: &Vec<Option<Transition>>,
        symbol: char,
    ) -> Option<&Transition> {
        let transition = transitions_map.get(symbol as usize);
        if transition.is_none() {
            return None;
        }

        transition.unwrap().as_ref()
    }

    fn get_accept_nfa_state(&self, s: usize) -> Option<usize> {
        let nfa_state = self.dfa_to_accepted_nfa_state_mapping.get(s);

        if nfa_state.is_none() {
            return None;
        }

        let nfa_state = nfa_state.unwrap();
        if nfa_state.is_none() {
            return None;
        }

        Some(nfa_state.clone().unwrap().0)
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
                let nfa_state = self.dfa_to_accepted_nfa_state_mapping.get(current_state.0);

                if nfa_state.is_none() {
                    println!("[WARN] This should only happen when the DFA is created from scratch, not created from NFA(s)");
                    return (None, true);
                }

                let nfa_state = self
                    .dfa_to_accepted_nfa_state_mapping
                    .get(current_state.0)
                    .unwrap();

                assert_eq!(nfa_state.is_some(), true);
                return (Some(nfa_state.clone().unwrap().0), true);
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
}

impl DFA {
    pub fn get_next_state(&self, state: State, c: u8) -> Option<State> {
        let transitions = &self.transitions[state.0];
        if 128 <= c {
            return None;
        }
        match &transitions[c as usize] {
            Some(transition) => Some(transition.to_state.clone()),
            None => None,
        }
    }

    pub fn is_accept_state(&self, state: State) -> Option<usize> {
        self.get_accept_nfa_state(state.0)
    }

    pub fn get_root(&self) -> State {
        self.start.clone()
    }
}

impl DFA {
    pub fn from_multiple_nfas(nfas: Vec<NFA>) -> DFA {
        // All of the nodes now have a pair of identifiers,
        // 1. the NFA index within the list of NFAs
        // 2. the NFA state index within the NFA

        // variables to create a new DFA
        let mut dfa_states: Vec<State> = Vec::new();
        let mut dfa_to_nfa_state_mapping: Vec<Rc<Vec<(usize, crate::nfa::nfa::State)>>> =
            Vec::new();
        let mut dfa_to_accepted_nfa_state_mapping: Vec<Option<(usize, crate::nfa::nfa::State)>> =
            Vec::new();
        let mut dfa_accept_states = HashSet::new();
        let mut dfa_transitions: Vec<Vec<Option<Transition>>> = Vec::new();

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

        let mut transition_vector = Vec::with_capacity(128);
        for _ in 0..128 {
            transition_vector.push(None::<Transition>);
        }
        dfa_transitions.push(transition_vector);

        dfa_to_nfa_state_mapping.push(start_epi_closure.clone());
        dfa_to_accepted_nfa_state_mapping.push(None);
        l_nfa_states_to_dfa_mapping.insert(start_epi_closure, State(start_state));
        l_worklist.push(State(start_state));

        // Process and add all dfa states
        while let Some(dfa_state) = l_worklist.pop() {
            // Take the immutable borrow into a local variable
            let nfa_states = { dfa_to_nfa_state_mapping.get(dfa_state.0).unwrap().clone() };

            // Check if this DFA state is an accept state
            for (idx, nfa_state) in nfa_states.iter() {
                if nfas.get(*idx).unwrap().get_accept() == *nfa_state {
                    dfa_to_accepted_nfa_state_mapping
                        .get_mut(dfa_state.0)
                        .as_mut()
                        .unwrap()
                        .replace((*idx, nfa_state.clone()));
                    dfa_accept_states.insert(dfa_state.clone());
                }
            }

            // Process the Move operation for all transitions in the NFA states set
            let mut move_transitions_symbol_to_transitions_vec = vec![Vec::new(); 128];
            for (idx, nfa_state) in nfa_states.iter() {
                let transitions = nfas
                    .get(*idx)
                    .unwrap()
                    .get_transitions_from_state(nfa_state);
                for transition in transitions.into_iter().flatten() {
                    let symbol_onehot_encoding = transition.get_symbol_onehot_encoding();

                    for i in 0..128 {
                        // We don't want to track epsilon transitions
                        if (symbol_onehot_encoding & (1 << i)) != 0 {
                            move_transitions_symbol_to_transitions_vec
                                .get_mut(i)
                                .unwrap()
                                .push((idx, transition));
                        }
                    }
                }
            }

            // Process the Epsilon Closure of the Move operation
            for (symbol, transitions) in move_transitions_symbol_to_transitions_vec
                .iter()
                .enumerate()
            {
                if transitions.is_empty() {
                    continue;
                }

                // Collect all the destination NFA states
                let mut destination_nfa_states = Vec::new();
                for (idx, transition) in transitions.iter() {
                    destination_nfa_states.push((**idx, (**transition).get_to_state()));
                }
                let destination_nfa_states =
                    Rc::new(DFA::epsilon_closure(&nfas, &destination_nfa_states));

                // Check if the destination NFA states are already in the DFA states set
                if !l_nfa_states_to_dfa_mapping.contains_key(&destination_nfa_states) {
                    // Add a new state to the DFA
                    let destination_dfa_state_idx = dfa_states.len();

                    dfa_states.push(State(destination_dfa_state_idx));
                    let mut transition_vector = Vec::new();
                    for _ in 0..128 {
                        transition_vector.push(None::<Transition>);
                    }
                    dfa_transitions.push(transition_vector);
                    dfa_to_accepted_nfa_state_mapping.push(None);

                    // Ensure no mutable and immutable borrow overlap
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

                // Add the transition to the DFA
                dfa_transitions.get_mut(dfa_state.0).unwrap()[symbol] = Some(Transition {
                    from_state: dfa_state.clone(),
                    symbol_onehot_encoding:
                        crate::nfa::nfa::Transition::convert_char_to_symbol_onehot_encoding(
                            symbol as u8 as char,
                        ),
                    to_state: destination_dfa_state.clone(),
                    tag: None,
                });
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

impl DfaSimulator {
    pub fn new(dfa: Rc<DFA>) -> Self {
        DfaSimulator {
            dfa: dfa.clone(),
            current_state: dfa.start.clone(),
        }
    }

    pub fn reset_simulation(&mut self) {
        self.current_state = self.dfa.start.clone();
    }

    // Simulate the DFA with a single character
    // Returns the next state and whether the current state is a valid state
    // invalid state means that the DFA has reached a dead end
    pub fn simulate_single_char(&mut self, input: char) -> (Option<usize>, bool) {
        let transitions = self.dfa.transitions.get(self.current_state.0);

        if transitions.is_none() {
            // not matched, nor is tracked by DFA, so invalid state
            return (None, false);
        }

        let transitions = transitions.unwrap();
        let transition = DFA::get_transition(transitions, input);

        if transition.is_none() {
            // not matched, nor is tracked by DFA, so invalid state
            return (None, false);
        }

        let next_state = transition.unwrap().to_state.clone();

        let potential_accept_state = self.dfa.get_accept_nfa_state(next_state.0);
        self.current_state = next_state;

        if potential_accept_state.is_some() {
            // we have a match
            return (potential_accept_state, true);
        }

        // not matched, but still valid
        (None, true)
    }
}
#[cfg(test)]
mod tests {
    use crate::dfa::dfa::{State, DFA};
    use crate::error_handling::Result;
    use crate::nfa::nfa::NFA;
    use crate::parser::regex_parser::parser::RegexParser;
    use crate::{dfa, nfa};
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn test_dfa() {
        let start = dfa::dfa::State(0);
        let accept = dfa::dfa::State(1);
        let mut dfa = DFA::new();

        dfa.states.push(accept.clone());
        let mut accept_transition_vec = Vec::new();
        for _ in 0..128 {
            accept_transition_vec.push(None);
        }
        dfa.transitions.push(accept_transition_vec);
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

    fn create_nfa1() -> Result<NFA> {
        // Should only match "a" or "ab"
        let mut parser = RegexParser::new();
        let parsed_ast = parser.parse_into_ast("(a)|(ab)")?;

        let mut nfa = NFA::new();
        nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;

        Ok(nfa)
    }

    fn create_nfa2() -> Result<NFA> {
        // Should match "c*"
        let mut parser = RegexParser::new();
        let parsed_ast = parser.parse_into_ast("c*")?;

        let mut nfa = NFA::new();
        nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;

        Ok(nfa)
    }

    fn create_nfa3() -> crate::error_handling::Result<NFA> {
        // Should match "c+ab"
        let mut parser = RegexParser::new();
        let parsed_ast = parser.parse_into_ast("c+ab")?;

        let mut nfa = NFA::new();
        nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;

        Ok(nfa)
    }

    #[test]
    fn test_nfa1_from_nfa_to_dfa() -> Result<()> {
        let nfa = create_nfa1()?;
        let dfa = DFA::from_multiple_nfas(vec![nfa]);

        print!("{:?}", dfa);

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
        let mut valid_transitions_count = 0;
        for transition in transitions_from_start.iter() {
            if transition.is_some() {
                valid_transitions_count += 1;
            }
        }
        assert_eq!(valid_transitions_count, 1);
        let transitions_from_start_given_a = transitions_from_start.get('a' as usize).unwrap();
        assert_eq!(
            transitions_from_start_given_a.as_ref().unwrap().to_state,
            State(1)
        );

        let transitions_to_accept = dfa.transitions.get(1).unwrap();
        let mut valid_transitions_count = 0;
        for transition in transitions_to_accept.iter() {
            if transition.is_some() {
                valid_transitions_count += 1;
            }
        }
        assert_eq!(valid_transitions_count, 1);
        let transitions_to_accept_given_b = transitions_to_accept.get('b' as usize).unwrap();
        assert_eq!(
            transitions_to_accept_given_b.as_ref().unwrap().to_state,
            State(2)
        );

        // Check correctness given some examples
        assert_eq!(dfa.simulate("a"), (Some(0usize), true));
        assert_eq!(dfa.simulate("ab"), (Some(0usize), true));
        assert_eq!(dfa.simulate("aa"), (None, false));
        assert_eq!(dfa.simulate("abb"), (None, false));
        assert_eq!(dfa.simulate("aba"), (None, false));

        Ok(())
    }

    #[test]
    fn test_nfa2_from_nfa_to_dfa() -> crate::error_handling::Result<()> {
        let nfa = create_nfa2()?;
        println!("{:?}", nfa);
        let dfa = DFA::from_multiple_nfas(vec![nfa]);

        // Check correctness given some examples
        assert_eq!(dfa.simulate("c"), (Some(0usize), true));
        assert_eq!(dfa.simulate("cc"), (Some(0usize), true));
        assert_eq!(dfa.simulate("ccc"), (Some(0usize), true));
        assert_eq!(dfa.simulate("cccc"), (Some(0usize), true));
        assert_eq!(dfa.simulate("ccccab"), (None, false));
        assert_eq!(dfa.simulate("cab"), (None, false));
        assert_eq!(dfa.simulate(""), (Some(0usize), true));

        Ok(())
    }

    #[test]
    fn test_nfa3_from_nfa_to_dfa() -> Result<()> {
        let nfa = create_nfa3()?;
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

        Ok(())
    }

    #[test]
    fn test_easy_from_multi_nfas_to_dfa() -> Result<()> {
        let nfa1 = create_nfa1()?;
        let nfa2 = create_nfa2()?;
        let nfa3 = create_nfa3()?;

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

        Ok(())
    }

    #[test]
    fn test_esay_from_multi_nfas_to_dfa_single_char_simulation() -> Result<()> {
        let nfa1 = create_nfa1()?;
        let nfa2 = create_nfa2()?;
        let nfa3 = create_nfa3()?;

        let dfa = DFA::from_multiple_nfas(vec![nfa1, nfa2, nfa3]);

        // Check correctness given some examples
        // Should match:
        // "a" or "ab"
        // "c*"
        // "c+ab"
        let mut dfa_simulator = dfa::dfa::DfaSimulator::new(Rc::new(dfa));
        assert_eq!(
            dfa_simulator.simulate_single_char('a'),
            (Some(0usize), true)
        );
        assert_eq!(
            dfa_simulator.simulate_single_char('b'),
            (Some(0usize), true)
        );
        assert_eq!(dfa_simulator.simulate_single_char('b'), (None, false));

        dfa_simulator.reset_simulation();
        assert_eq!(
            dfa_simulator.simulate_single_char('c'),
            (Some(1usize), true)
        );
        assert_eq!(
            dfa_simulator.simulate_single_char('c'),
            (Some(1usize), true)
        );
        assert_eq!(
            dfa_simulator.simulate_single_char('c'),
            (Some(1usize), true)
        );
        assert_eq!(dfa_simulator.simulate_single_char('a'), (None, true));
        assert_eq!(
            dfa_simulator.simulate_single_char('b'),
            (Some(2usize), true)
        );

        dfa_simulator.reset_simulation();
        assert_eq!(
            dfa_simulator.simulate_single_char('c'),
            (Some(1usize), true)
        );
        assert_eq!(dfa_simulator.simulate_single_char('b'), (None, false));

        Ok(())
    }

    #[test]
    fn test_int() -> Result<()> {
        let mut parser = RegexParser::new();
        let parsed_ast = parser.parse_into_ast(r"\-{0,1}\d+")?;

        let mut nfa = NFA::new();
        nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;

        let dfa = DFA::from_multiple_nfas(vec![nfa]);

        assert_eq!(dfa.simulate("0"), (Some(0usize), true));
        assert_eq!(dfa.simulate("1234"), (Some(0usize), true));
        assert_eq!(dfa.simulate("-1234"), (Some(0usize), true));
        assert_eq!(dfa.simulate("-0"), (Some(0usize), true));
        assert_eq!(dfa.simulate("aba"), (None, false));
        assert_eq!(dfa.simulate(""), (None, false));
        assert_eq!(dfa.simulate("3.14"), (None, false));
        assert_eq!(dfa.simulate("0.00"), (None, false));

        Ok(())
    }

    #[test]
    fn test_float() -> Result<()> {
        let mut parser = RegexParser::new();
        let parsed_ast = parser.parse_into_ast(r"\-{0,1}[0-9]+\.\d+")?;

        let mut nfa = NFA::new();
        nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;

        let dfa = DFA::from_multiple_nfas(vec![nfa]);

        assert_eq!(dfa.simulate("0.0"), (Some(0usize), true));
        assert_eq!(dfa.simulate("-0.0"), (Some(0usize), true));
        assert_eq!(dfa.simulate("-0.00001"), (Some(0usize), true));
        assert_eq!(dfa.simulate("0.00001"), (Some(0usize), true));
        assert_eq!(dfa.simulate("3.1415926"), (Some(0usize), true));
        assert_eq!(dfa.simulate("-3.1415926"), (Some(0usize), true));

        assert_eq!(dfa.simulate("0"), (None, false));
        assert_eq!(dfa.simulate("1234"), (None, false));
        assert_eq!(dfa.simulate("-1234"), (None, false));
        assert_eq!(dfa.simulate("-0"), (None, false));
        assert_eq!(dfa.simulate("aba"), (None, false));
        assert_eq!(dfa.simulate(""), (None, false));

        Ok(())
    }

    #[test]
    fn test_hex() -> Result<()> {
        let mut parser = RegexParser::new();
        let parsed_ast = parser.parse_into_ast(r"(0x){0,1}(((\d|[a-f])+)|((\d|[A-F])+))")?;

        let mut nfa = NFA::new();
        nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
        println!("{:?}", nfa);

        let dfa = DFA::from_multiple_nfas(vec![nfa]);
        println!("{:?}", dfa);

        assert_eq!(dfa.simulate("0x0"), (Some(0usize), true));
        assert_eq!(dfa.simulate("0"), (Some(0usize), true));
        assert_eq!(dfa.simulate("1234"), (Some(0usize), true));
        assert_eq!(dfa.simulate("0x1A2B3C4D5E6F7890"), (Some(0usize), true));
        assert_eq!(dfa.simulate("0x1a2b3c4d5e6f7890"), (Some(0usize), true));
        assert_eq!(
            dfa.simulate("0xddba9b95eeb3cfb9ccb3d8401d1610d42f0e3aad"),
            (Some(0usize), true)
        );

        assert_eq!(dfa.simulate("1a2b3c4d5e6f7890"), (Some(0usize), true));
        assert_eq!(dfa.simulate("abcdef"), (Some(0usize), true));
        assert_eq!(dfa.simulate("abcdefg"), (None, false));
        assert_eq!(dfa.simulate("aBa"), (None, false));
        assert_eq!(dfa.simulate(""), (None, false));
        assert_eq!(dfa.simulate("3.14"), (None, false));
        assert_eq!(dfa.simulate("0.00"), (None, false));

        Ok(())
    }

    #[test]
    fn test_timestamp() -> Result<()> {
        let mut parser = RegexParser::new();
        let parsed_ast = parser.parse_into_ast(r"\d{4}\-\d{2}\-\d{2}T\d{2}:\d{2}:\d{2}\.\d{2}")?;

        let mut nfa = NFA::new();
        nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
        println!("{:?}", nfa);

        let dfa = DFA::from_multiple_nfas(vec![nfa]);
        println!("{:?}", dfa);

        assert_eq!(dfa.simulate("2015-01-31T15:50:45.39"), (Some(0usize), true));

        Ok(())
    }

    #[test]
    fn test_static_text() -> Result<()> {
        let mut parser = RegexParser::new();
        let parsed_ast = parser.parse_into_ast(r"TIMESTAMP")?;

        let mut nfa = NFA::new();
        nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
        println!("{:?}", nfa);

        let dfa = DFA::from_multiple_nfas(vec![nfa]);
        println!("{:?}", dfa);

        assert_eq!(dfa.simulate("TIMESTAMP"), (Some(0usize), true));
        assert_eq!(dfa.simulate("This log "), (None, false));

        Ok(())
    }

    #[test]
    fn test_repetition() -> Result<()> {
        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"a{0,3}")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
            println!("{:?}", nfa);

            let dfa = DFA::from_multiple_nfas(vec![nfa]);
            println!("{:?}", dfa);

            assert_eq!(dfa.simulate(""), (Some(0usize), true));
            assert_eq!(dfa.simulate("a"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaa"), (None, false));
        }

        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"a{0,1}")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
            println!("{:?}", nfa);

            let dfa = DFA::from_multiple_nfas(vec![nfa]);
            println!("{:?}", dfa);

            assert_eq!(dfa.simulate(""), (Some(0usize), true));
            assert_eq!(dfa.simulate("a"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aa"), (None, false));
        }

        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"a*")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
            println!("{:?}", nfa);

            let dfa = DFA::from_multiple_nfas(vec![nfa]);
            println!("{:?}", dfa);

            assert_eq!(dfa.simulate(""), (Some(0usize), true));
            assert_eq!(dfa.simulate("a"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaaaaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("ab"), (None, false));
            assert_eq!(dfa.simulate("ba"), (None, false));
        }

        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"a+")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
            println!("{:?}", nfa);

            let dfa = DFA::from_multiple_nfas(vec![nfa]);
            println!("{:?}", dfa);

            assert_eq!(dfa.simulate(""), (None, false));
            assert_eq!(dfa.simulate("a"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaaaaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("ab"), (None, false));
            assert_eq!(dfa.simulate("ba"), (None, false));
        }

        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"a{1,}")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
            println!("{:?}", nfa);

            let dfa = DFA::from_multiple_nfas(vec![nfa]);
            println!("{:?}", dfa);

            assert_eq!(dfa.simulate(""), (None, false));
            assert_eq!(dfa.simulate("a"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaaaaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("ab"), (None, false));
            assert_eq!(dfa.simulate("ba"), (None, false));
        }

        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"a{3,}")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
            println!("{:?}", nfa);

            let dfa = DFA::from_multiple_nfas(vec![nfa]);
            println!("{:?}", dfa);

            assert_eq!(dfa.simulate(""), (None, false));
            assert_eq!(dfa.simulate("a"), (None, false));
            assert_eq!(dfa.simulate("aa"), (None, false));
            assert_eq!(dfa.simulate("aaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaaaaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("ab"), (None, false));
            assert_eq!(dfa.simulate("ba"), (None, false));
        }

        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"a{3}")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
            println!("{:?}", nfa);

            let dfa = DFA::from_multiple_nfas(vec![nfa]);
            println!("{:?}", dfa);

            assert_eq!(dfa.simulate(""), (None, false));
            assert_eq!(dfa.simulate("a"), (None, false));
            assert_eq!(dfa.simulate("aa"), (None, false));
            assert_eq!(dfa.simulate("aaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaa"), (None, false));
            assert_eq!(dfa.simulate("aaaaaaaa"), (None, false));
            assert_eq!(dfa.simulate("ab"), (None, false));
            assert_eq!(dfa.simulate("ba"), (None, false));
        }

        {
            let mut parser = RegexParser::new();
            let parsed_ast = parser.parse_into_ast(r"a{3,6}")?;

            let mut nfa = NFA::new();
            nfa.add_ast_to_nfa(&parsed_ast, NFA::START_STATE, NFA::ACCEPT_STATE)?;
            println!("{:?}", nfa);

            let dfa = DFA::from_multiple_nfas(vec![nfa]);
            println!("{:?}", dfa);

            assert_eq!(dfa.simulate(""), (None, false));
            assert_eq!(dfa.simulate("a"), (None, false));
            assert_eq!(dfa.simulate("aa"), (None, false));
            assert_eq!(dfa.simulate("aaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaaaa"), (Some(0usize), true));
            assert_eq!(dfa.simulate("aaaaaaa"), (None, false));
            assert_eq!(dfa.simulate("aaaaaaaa"), (None, false));
            assert_eq!(dfa.simulate("ab"), (None, false));
            assert_eq!(dfa.simulate("ba"), (None, false));
        }

        Ok(())
    }
}
