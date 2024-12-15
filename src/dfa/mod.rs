pub(crate) mod dfa;

#[cfg(feature = "regex-engine")]
pub use dfa::DfaSimulator;
#[cfg(feature = "regex-engine")]
pub use dfa::State;
#[cfg(feature = "regex-engine")]
pub use dfa::DFA;
