use crate::position::Position;

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
