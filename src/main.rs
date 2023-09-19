extern crate bkgm;

use bkgm::State;

const ROLLS: [(usize, usize); 21] = [
    (1, 1),
    (1, 2),
    (1, 3),
    (1, 4),
    (1, 5),
    (1, 6),
    (2, 2),
    (2, 3),
    (2, 4),
    (2, 5),
    (2, 6),
    (3, 3),
    (3, 4),
    (3, 5),
    (3, 6),
    (4, 4),
    (4, 5),
    (4, 6),
    (5, 5),
    (5, 6),
    (6, 6),
];

fn perft_rec(mut state: &mut State, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut count = 0;

    for roll in ROLLS {
        for action in state.possible_actions(roll) {
            state.apply_action(&action);
            count += perft_rec(&mut state, depth - 1);
            state.undo_action(&action);
        }
    }

    count
}

fn perft(depth: usize) {
    let mut state = State::new();
    let mut count = 0;

    for roll in ROLLS {
        let mut roll_count = 0;
        for action in state.possible_actions(roll) {
            state.apply_action(&action);
            roll_count += perft_rec(&mut state, depth - 1);
            state.undo_action(&action);
        }
        count += roll_count;
        println!("{:?}: {}", roll, roll_count);
    }

    println!("{} positions", count);
}

fn main() {
    perft(2);
}
