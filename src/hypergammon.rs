use crate::position::{Position, State};
use crate::utils::mcomb;
use crate::{hpos, Dice};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hypergammon {
    position: Position,
}

impl State for Hypergammon {
    const NUM_CHECKERS: u8 = 3;

    fn new() -> Self {
        hpos!(x 24:1, 23:1, 22:1; o 1:1, 2:1, 3:1)
    }

    fn from_position(position: Position) -> Self {
        Self { position }
    }

    #[inline(always)]
    fn x_bar(&self) -> u8 {
        self.position.x_bar()
    }

    #[inline(always)]
    fn o_bar(&self) -> u8 {
        self.position.o_bar()
    }

    #[inline(always)]
    fn x_off(&self) -> u8 {
        self.position.x_off()
    }

    #[inline(always)]
    fn o_off(&self) -> u8 {
        self.position.o_off()
    }

    #[inline(always)]
    fn pip(&self, pip: usize) -> i8 {
        self.position.pip(pip)
    }

    #[inline(always)]
    fn board(&self) -> [i8; 24] {
        // self.position.board()
        [0; 24]
    }

    #[inline(always)]
    fn position(&self) -> Position {
        self.position
    }

    fn flip(&self) -> Self {
        Self {
            position: self.position.flip(),
        }
    }

    fn dbhash(&self) -> usize {
        let points = 26;
        let mut x_remaining = 3 - self.position.x_off() as usize;
        let mut o_remaining = 3 - self.position.o_off() as usize;
        let mut x_index = if x_remaining > 0 {
            mcomb(points, x_remaining - 1)
        } else {
            0
        };
        let mut o_index = if o_remaining > 0 {
            mcomb(points, o_remaining - 1)
        } else {
            0
        };
        for (i, &n) in self.position.pips.iter().skip(1).enumerate() {
            if n > 0 {
                x_remaining -= n as usize;
            } else if n < 0 {
                o_remaining -= n.abs() as usize;
            }
            if o_remaining > 0 {
                o_index += mcomb(points - 1 - i, o_remaining - 1);
            }
            if x_remaining > 0 {
                x_index += mcomb(points - 1 - i, x_remaining - 1);
            }
        }
        x_index * mcomb(points, 3) + o_index
    }
}

/// Simple way to create positions for testing
/// The starting position would be:
/// pos!(x 24:2, 13:5, 8:3, 6:5; o 19:5, 17:3, 12:5, 1:2)
/// The order is not important, so this is equivalent:
/// pos!(x 24:2, 13:5, 8:3, 6:5; o 1:2, 12:5, 17:3, 19:5)
/// TODO: change macro to remove X_BAR and O_BAR
#[macro_export]
macro_rules! hpos {
    ( x $( $x_pip:tt : $x_checkers:tt ), * ;o $( $o_pip:tt : $o_checkers:tt ), * ) => {
        {
            use crate::position::Position;
            #[allow(unused_mut)]
            let mut pips = [0; 26];
            let mut x_pieces = 0;
            let mut o_pieces = 0;

            $(
                pips[$x_pip as usize] = $x_checkers as i8;
                x_pieces += $x_checkers;
            )*

            $(
                pips[$o_pip as usize] = -$o_checkers as i8;
                o_pieces += $o_checkers;
            )*

            let x_off = 3 - x_pieces;
            let o_off = 3 - o_pieces;

            let position = Position {
                pips,
                x_off,
                o_off,
            };

            Hypergammon::from_position(position)
        }
    };
}

#[cfg(test)]
mod hypergammon_test {
    use crate::{
        position::{Position, State},
        Hypergammon,
    };

    #[test]
    fn start_positon() {
        let start = Hypergammon::new();
        let expected = Hypergammon {
            position: Position {
                pips: [
                    0, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0,
                ],
                x_off: 0,
                o_off: 0,
            },
        };
        assert_eq!(start, expected);
    }

    #[test]
    fn flip() {
        let position = hpos!(x 1:1, 3:1, 18:1; o 19:1, 5:1);
        let expected = hpos!(x 6:1, 20:1; o 24:1, 22:1, 7:1);
        assert_eq!(position.flip(), expected);
    }
}
