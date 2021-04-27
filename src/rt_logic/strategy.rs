//! Strategies to determine which stations to actually visit.
//!
//! Summaries of strategies:
//!
//! * [`IntegrityStrat`]:
//!
//!     Checks each station and visits it if it is not already in the desired
//!     state.
//!
//! * `MinWorkStrat` *(not yet implemented)*:
//!
//!     Works backwards from the final destination, and only visits dependencies
//!     if the current station is not in the desired state.

pub use self::integrity_strat::IntegrityStrat;

mod integrity_strat;
