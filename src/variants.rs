use crate::dice::Dice;
use crate::position::Position;
use crate::position::{GamePhase, GameState, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    Backgammon,
    Nackgammon,
    Longgammon,
    Hypergammon,
    Hypergammon2,
    Hypergammon4,
    Hypergammon5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariantPosition {
    Backgammon(Position<15>),
    Nackgammon(Position<15>),
    Longgammon(Position<15>),
    Hypergammon(Position<3>),
    Hypergammon2(Position<2>),
    Hypergammon4(Position<4>),
    Hypergammon5(Position<5>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariantSpec {
    pub variant: Variant,
    pub name: &'static str,
    pub num_checkers: u8,
    pub start: VariantPosition,
}

impl Variant {
    pub fn spec(self) -> VariantSpec {
        match self {
            Variant::Backgammon => VariantSpec {
                variant: self,
                name: "Backgammon",
                num_checkers: 15,
                start: VariantPosition::Backgammon(BACKGAMMON),
            },
            Variant::Nackgammon => VariantSpec {
                variant: self,
                name: "Nackgammon",
                num_checkers: 15,
                start: VariantPosition::Nackgammon(NACKGAMMON),
            },
            Variant::Longgammon => VariantSpec {
                variant: self,
                name: "Longgammon",
                num_checkers: 15,
                start: VariantPosition::Longgammon(LONGGAMMON),
            },
            Variant::Hypergammon => VariantSpec {
                variant: self,
                name: "Hypergammon (3)",
                num_checkers: 3,
                start: VariantPosition::Hypergammon(HYPERGAMMON),
            },
            Variant::Hypergammon2 => VariantSpec {
                variant: self,
                name: "Hypergammon (2)",
                num_checkers: 2,
                start: VariantPosition::Hypergammon2(HYPERGAMMON2),
            },
            Variant::Hypergammon4 => VariantSpec {
                variant: self,
                name: "Hypergammon (4)",
                num_checkers: 4,
                start: VariantPosition::Hypergammon4(HYPERGAMMON4),
            },
            Variant::Hypergammon5 => VariantSpec {
                variant: self,
                name: "Hypergammon (5)",
                num_checkers: 5,
                start: VariantPosition::Hypergammon5(HYPERGAMMON5),
            },
        }
    }

    pub fn start_position(self) -> VariantPosition {
        self.spec().start
    }

    pub fn from_position_id(self, id: &str) -> Option<VariantPosition> {
        match self {
            Variant::Backgammon | Variant::Nackgammon | Variant::Longgammon => {
                <Position<15> as State>::from_id(id).map(|p| match self {
                    Variant::Backgammon => VariantPosition::Backgammon(p),
                    Variant::Nackgammon => VariantPosition::Nackgammon(p),
                    Variant::Longgammon => VariantPosition::Longgammon(p),
                    _ => unreachable!(),
                })
            }
            Variant::Hypergammon => {
                <Position<3> as State>::from_id(id).map(VariantPosition::Hypergammon)
            }
            Variant::Hypergammon2 => {
                <Position<2> as State>::from_id(id).map(VariantPosition::Hypergammon2)
            }
            Variant::Hypergammon4 => {
                <Position<4> as State>::from_id(id).map(VariantPosition::Hypergammon4)
            }
            Variant::Hypergammon5 => {
                <Position<5> as State>::from_id(id).map(VariantPosition::Hypergammon5)
            }
        }
    }
}

impl VariantPosition {
    pub fn variant(self) -> Variant {
        match self {
            VariantPosition::Backgammon(_) => Variant::Backgammon,
            VariantPosition::Nackgammon(_) => Variant::Nackgammon,
            VariantPosition::Longgammon(_) => Variant::Longgammon,
            VariantPosition::Hypergammon(_) => Variant::Hypergammon,
            VariantPosition::Hypergammon2(_) => Variant::Hypergammon2,
            VariantPosition::Hypergammon4(_) => Variant::Hypergammon4,
            VariantPosition::Hypergammon5(_) => Variant::Hypergammon5,
        }
    }

    pub fn num_checkers(self) -> u8 {
        match self {
            VariantPosition::Backgammon(_) => 15,
            VariantPosition::Nackgammon(_) => 15,
            VariantPosition::Longgammon(_) => 15,
            VariantPosition::Hypergammon(_) => 3,
            VariantPosition::Hypergammon2(_) => 2,
            VariantPosition::Hypergammon4(_) => 4,
            VariantPosition::Hypergammon5(_) => 5,
        }
    }

    pub fn turn(self) -> bool {
        match self {
            VariantPosition::Backgammon(p) => p.turn(),
            VariantPosition::Nackgammon(p) => p.turn(),
            VariantPosition::Longgammon(p) => p.turn(),
            VariantPosition::Hypergammon(p) => p.turn(),
            VariantPosition::Hypergammon2(p) => p.turn(),
            VariantPosition::Hypergammon4(p) => p.turn(),
            VariantPosition::Hypergammon5(p) => p.turn(),
        }
    }

    pub fn possible_positions(self, dice: &Dice) -> Vec<Self> {
        match self {
            VariantPosition::Backgammon(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Backgammon)
                .collect(),
            VariantPosition::Nackgammon(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Nackgammon)
                .collect(),
            VariantPosition::Longgammon(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Longgammon)
                .collect(),
            VariantPosition::Hypergammon(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Hypergammon)
                .collect(),
            VariantPosition::Hypergammon2(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Hypergammon2)
                .collect(),
            VariantPosition::Hypergammon4(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Hypergammon4)
                .collect(),
            VariantPosition::Hypergammon5(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Hypergammon5)
                .collect(),
        }
    }

    pub fn phase(self) -> GamePhase {
        match self {
            VariantPosition::Backgammon(p) => p.phase(),
            VariantPosition::Nackgammon(p) => p.phase(),
            VariantPosition::Longgammon(p) => p.phase(),
            VariantPosition::Hypergammon(p) => p.phase(),
            VariantPosition::Hypergammon2(p) => p.phase(),
            VariantPosition::Hypergammon4(p) => p.phase(),
            VariantPosition::Hypergammon5(p) => p.phase(),
        }
    }

    pub fn game_state(self) -> GameState {
        match self {
            VariantPosition::Backgammon(p) => p.game_state(),
            VariantPosition::Nackgammon(p) => p.game_state(),
            VariantPosition::Longgammon(p) => p.game_state(),
            VariantPosition::Hypergammon(p) => p.game_state(),
            VariantPosition::Hypergammon2(p) => p.game_state(),
            VariantPosition::Hypergammon4(p) => p.game_state(),
            VariantPosition::Hypergammon5(p) => p.game_state(),
        }
    }

    pub fn position_id(self) -> String {
        match self {
            VariantPosition::Backgammon(p) => p.position_id(),
            VariantPosition::Nackgammon(p) => p.position_id(),
            VariantPosition::Longgammon(p) => p.position_id(),
            VariantPosition::Hypergammon(p) => p.position_id(),
            VariantPosition::Hypergammon2(p) => p.position_id(),
            VariantPosition::Hypergammon4(p) => p.position_id(),
            VariantPosition::Hypergammon5(p) => p.position_id(),
        }
    }
}

pub const BACKGAMMON: Position<15> = Position {
    turn: true,
    pips: [
        0, -2, 0, 0, 0, 0, 5, 0, 3, 0, 0, 0, -5, 5, 0, 0, 0, -3, 0, -5, 0, 0, 0, 0, 2, 0,
    ],
    x_off: 0,
    o_off: 0,
};

pub const NACKGAMMON: Position<15> = Position {
    turn: true,
    pips: [
        0, -2, -2, 0, 0, 0, 4, 0, 3, 0, 0, 0, -4, 4, 0, 0, 0, -3, 0, -4, 0, 0, 0, 2, 2, 0,
    ],
    x_off: 0,
    o_off: 0,
};

pub const HYPERGAMMON: Position<3> = Position {
    turn: true,
    pips: [
        0, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0,
    ],
    x_off: 0,
    o_off: 0,
};

pub const HYPERGAMMON2: Position<2> = Position {
    turn: true,
    pips: [
        0, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0,
    ],
    x_off: 0,
    o_off: 0,
};

pub const HYPERGAMMON4: Position<4> = Position {
    turn: true,
    pips: [
        0, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0,
    ],
    x_off: 0,
    o_off: 0,
};

pub const HYPERGAMMON5: Position<5> = Position {
    turn: true,
    pips: [
        0, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0,
    ],
    x_off: 0,
    o_off: 0,
};

pub const LONGGAMMON: Position<15> = Position {
    turn: true,
    pips: [
        0, -15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0,
    ],
    x_off: 0,
    o_off: 0,
};
