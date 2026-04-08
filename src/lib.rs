pub mod dice;
pub mod dice_gen;
pub mod game;
pub mod position;
pub mod rules;
pub mod utils;
pub mod variants;
pub mod xgid;

pub use dice::Dice;
pub use game::Game;
pub use position::{GameResult, GameState, Position, State, O_BAR, X_BAR};
pub use rules::{
    legal_positions, legal_positions_with, ClassicRules, NoHitRules, PositionRules, VariantRules,
};
pub use variants::*;
pub use xgid::{Xgid, XgidBoard, XgidDice};

// pub use backgammon::Backgammon;
// pub use hypergammon::Hypergammon;
