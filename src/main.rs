// use std::collections::HashSet;

// use bkgm::{
//     dice::{ALL_21, ALL_SINGLES},
//     Dice, GameState, Hypergammon, State,
// };

// fn perft_rec(position: &Hypergammon, dice: &Dice, depth: u32) -> u64 {
//     let positions = position.possible_positions(dice);

//     if depth == 0 {
//         return positions.len() as u64;
//     }

//     let mut nodes = 0;
//     for (dice, _) in ALL_21 {
//         for position in positions.iter() {
//             nodes += match position.game_state() {
//                 GameState::Ongoing => perft_rec(position, &dice, depth - 1),
//                 GameState::GameOver(_) => 1,
//             }
//         }
//     }
//     nodes
// }

// fn perft(position: &Hypergammon, depth: u32) {
//     let mut total = 0;
//     for dice in ALL_SINGLES {
//         let nodes = perft_rec(&position, &dice, depth - 1);
//         total += nodes;
//         println!("{}: {}", dice, nodes);
//     }
//     println!("Total: {}", total);
// }

// fn unqiue(verbose: bool) {
//     let position = Hypergammon::new();
//     let mut found = HashSet::new();
//     let mut new_positons = vec![];
//     let before = found.len();

//     for die in ALL_SINGLES {
//         let children = position.possible_positions(&die);
//         for child in children {
//             if !found.contains(&child) {
//                 found.insert(child);
//                 new_positons.push(child);
//             }
//         }
//     }

//     let mut depth = 1;
//     let discovered = found.len() - before;
//     if verbose {
//         println!(
//             "{}\t{}\tpositions reached after {} roll",
//             discovered,
//             found.len(),
//             depth
//         );
//     }

//     while !new_positons.is_empty() {
//         let mut queue = new_positons;
//         new_positons = vec![];
//         let before = found.len();
//         while let Some(position) = queue.pop() {
//             match position.game_state() {
//                 GameState::Ongoing => {
//                     for (die, _) in ALL_21 {
//                         let children = position.possible_positions(&die);
//                         for child in children {
//                             if !found.contains(&child) {
//                                 found.insert(child);
//                                 new_positons.push(child);
//                             }
//                         }
//                     }
//                 }
//                 GameState::GameOver(_) => {}
//             }
//         }
//         let discovered = found.len() - before;
//         depth += 1;
//         if verbose {
//             println!(
//                 "{}\t{}\tpositions reached after {} rolls",
//                 discovered,
//                 found.len(),
//                 depth
//             );
//         }
//     }

//     // found.into_iter().collect()
// }

// fn main() {
//     // let hyper = bkgm::Hypergammon::new();
//     // perft(&hyper, 1);

//     // println!("{}", hyper.position_id());

//     // let mut found = std::collections::HashSet::new();

//     // let position = bkgm::Hypergammon::new();
//     // for die in ALL_SINGLES {
//     //     let children = position.possible_positions(&die);
//     //     for child in children {
//     //         if !found.contains(&child) {
//     //             found.insert(child);
//     //         }
//     //     }
//     // }

//     // println!("Unique positions: {}", found.len());
//     unqiue(true);
// }

fn main() {}
