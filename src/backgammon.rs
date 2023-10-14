use crate::bpos;
use crate::position::{Position, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Backgammon {
    position: Position,
}

impl Backgammon {
    pub fn from_macro(pips: [i8; 26], x_off: u8, o_off: u8) -> Self {
        let position = Position { pips, x_off, o_off };
        Self::from_position(position)
    }
}

impl State for Backgammon {
    const NUM_CHECKERS: u8 = 15;

    fn new() -> Self {
        bpos!(x 24:2, 13:5, 8:3, 6:5; o 19:5, 17:3, 12:5, 1:2)
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
        todo!()
    }
}

/// Simple way to create positions for testing
/// The starting position would be:
/// pos!(x 24:2, 13:5, 8:3, 6:5; o 19:5, 17:3, 12:5, 1:2)
/// The order is not important, so this is equivalent:
/// pos!(x 24:2, 13:5, 8:3, 6:5; o 1:2, 12:5, 17:3, 19:5)
/// TODO: change macro to remove X_BAR and O_BAR
#[macro_export]
macro_rules! bpos {
    ( x $( $x_pip:tt : $x_checkers:tt ), * ;o $( $o_pip:tt : $o_checkers:tt ), * ) => {
        {
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

            let x_off = 15 - x_pieces;
            let o_off = 15 - o_pieces;


            Backgammon::from_macro(pips, x_off, o_off)
        }
    };
}

#[cfg(test)]
mod backgammon_test {
    use crate::{
        position::{Position, State},
        Backgammon,
        GameResult::*,
        GameState::*,
    };

    #[test]
    fn start_positon() {
        let start = Backgammon::new();
        let expected = Backgammon {
            position: Position {
                pips: [
                    0, -2, 0, 0, 0, 0, 5, 0, 3, 0, 0, 0, -5, 5, 0, 0, 0, -3, 0, -5, 0, 0, 0, 0, 2,
                    0,
                ],
                x_off: 0,
                o_off: 0,
            },
        };
        assert_eq!(start, expected);
    }

    #[test]
    fn flip() {
        let position = bpos!(x 24:2, 17:3; o 1:2, 18:5);
        let expected = bpos!(x 24:2, 7:5; o 1:2, 8:3);
        assert_eq!(position.flip(), expected);
    }

    #[test]
    fn game_state_bg_when_on_bar() {
        let given = bpos!(x 25:1, 1:14; o);
        assert_eq!(given.game_state(), GameOver(LoseBackgammon));
        assert_eq!(given.flip().game_state(), GameOver(WinBackgammon));
    }

    #[test]
    fn game_state_bg_when_not_on_bar() {
        let given = bpos!(x 19:15; o);
        assert_eq!(given.game_state(), GameOver(LoseBackgammon));
        assert_eq!(given.flip().game_state(), GameOver(WinBackgammon));
    }

    #[test]
    fn game_state_gammon() {
        let given = bpos!(x 18:15; o);
        assert_eq!(given.game_state(), GameOver(LoseGammon));
        assert_eq!(given.flip().game_state(), GameOver(LoseGammon.reverse()));
    }

    #[test]
    fn game_state_normal() {
        let given = bpos!(x 19:14; o);
        assert_eq!(given.game_state(), GameOver(LoseNormal));
        assert_eq!(given.flip().game_state(), GameOver(LoseNormal.reverse()));
    }

    #[test]
    fn game_state_ongoing() {
        let given = bpos!(x 19:14; o 1:4);
        assert_eq!(given.game_state(), Ongoing);
        assert_eq!(given.flip().game_state(), Ongoing);
    }
}
