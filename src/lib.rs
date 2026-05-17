pub mod bearoff;
pub mod codecs;
pub mod dice;
pub mod dice_gen;
pub mod engine_spec;
pub mod game;
pub mod position;
pub mod rules;
pub mod ubgi;
pub mod utils;
pub mod variants;

pub use codecs::move_text::{
    apply as apply_move, encode as encode_move, encode_steps as encode_move_steps,
    format_move_steps, legal as legal_moves, normalize as normalize_move_text, parse_move_steps,
    MoveStep,
};
pub use codecs::xgid::{Xgid, XgidBoard, XgidDice};
pub use dice::Dice;
pub use engine_spec::{format_engine_spec, parse_engine_spec, EngineSpec};
pub use game::Game;
pub use position::{GameResult, GameState, Position, State, O_BAR, X_BAR};
pub use rules::{
    legal_positions, legal_positions_with, ClassicRules, NoHitRules, PositionRules, VariantRules,
};
pub use variants::*;

// pub use backgammon::Backgammon;
// pub use hypergammon::Hypergammon;
