pub(crate) mod nfa;

#[cfg(feature = "regex-engine")]
pub use crate::nfa::nfa::State;

#[cfg(feature = "regex-engine")]
pub use crate::nfa::nfa::NFA;

#[cfg(feature = "regex-engine")]
pub use crate::nfa::nfa::Transition;
