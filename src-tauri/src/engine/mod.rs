//! Chess engine crate surface: board, search, Elo strength, analysis, and coach mode.

pub mod analysis;
pub mod board;
pub mod coach;
pub mod elo;
pub mod eval;
pub mod movegen;
pub mod search;
pub mod types;

pub use analysis::{analyze_move, MoveAnalysis};
pub use board::Board;
pub use elo::{choose_move, preset_levels, EloProfile};
pub use types::*;
