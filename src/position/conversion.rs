use crate::position::Position;
use std::collections::HashMap;

/// Simple way to create positions for testing.
///
/// The starting position would be:
/// pos!(x 24:2, 13:5, 8:3, 6:5; o 19:5, 17:3, 12:5, 1:2)
/// The order is not important, so this is equivalent:
/// pos!(x 24:2, 13:5, 8:3, 6:5; o 1:2, 12:5, 17:3, 19:5)
#[macro_export]
macro_rules! pos {
    ( x $( $x_pip:tt:$x_checkers:tt ), * ;o $( $o_pip:tt:$o_checkers:tt ), * ) => {
        {
            #[allow(unused_mut)]
            let mut x = std::collections::HashMap::new();
            $(
                x.insert($x_pip as usize, $x_checkers as u8);
            )*

            #[allow(unused_mut)]
            let mut o = std::collections::HashMap::new();
            $(
                o.insert($o_pip as usize, $o_checkers as u8);
            )*

            $crate::position::Position::<15>::from_hash_maps(&x, &o)
        }
    };
}

impl<const N: u8> Position<N> {
    pub fn from_hash_maps(x: &HashMap<usize, u8>, o: &HashMap<usize, u8>) -> Self {
        let mut pips = [0; 26];
        for (i, v) in x {
            pips[*i] = *v as i8;
        }
        for (i, v) in o {
            debug_assert!(pips[*i] == 0);
            pips[*i] = -(*v as i8);
        }
        Self::try_from(pips).expect("Need legal position")
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         games::BACKGAMMON,
//         position::{Position, O_BAR, X_BAR},
//     };

//     #[test]
//     fn start_id() {
//         let game = BACKGAMMON;
//         let id = game.position_id();
//         assert_eq!(id, "4HPwATDgc/ABMA");
//     }

//     #[test]
//     fn matching_ids() {
//         let pids = [
//             "4HPwATDgc/ABMA", // starting position
//             "jGfkASjg8wcBMA", // random position
//             "zGbiIQgxH/AAWA", // X bar
//             "zGbiIYCYD3gALA", // O off
//         ];
//         for pid in pids {
//             let game = Position::<15>::from_id(pid);
//             assert_eq!(pid, game.position_id());
//         }
//     }

//     #[test]
//     fn matching_positions() {
//         let pos1 = pos!(x 24:1, X_BAR:2; o 1:3, O_BAR: 4);
//         let pos2 = pos!(x 2:10, 1:5; o 24:9, 23:6);
//         for position in [pos1, pos2] {
//             let id = position.position_id();
//             assert_eq!(position, Position::from_id(&id));
//         }
//     }
// }
