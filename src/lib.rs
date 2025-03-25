pub mod dice;
pub mod dice_gen;
pub mod position;
pub mod utils;
pub mod variants;

pub use dice::Dice;
pub use position::{GameResult, GameState, Position, State, O_BAR, X_BAR};
pub use variants::*;

// pub use backgammon::Backgammon;
// pub use hypergammon::Hypergammon;
