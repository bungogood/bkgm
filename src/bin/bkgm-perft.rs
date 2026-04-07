use std::env;
use std::time::Instant;

use bkgm::dice::ALL_21;
use bkgm::{Dice, Position, State};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn parse_usize_flag(args: &[String], name: &str) -> Option<usize> {
    args.windows(2)
        .find(|w| w[0] == name)
        .and_then(|w| w[1].parse::<usize>().ok())
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn perft_all_rolls(
    position: &Position<15>,
    depth: usize,
    weighted: bool,
    scratch: &mut [Vec<Position<15>>],
) -> u128 {
    if depth == 0 {
        return 1;
    }

    let (lower_levels, this_level_and_up) = scratch.split_at_mut(depth);
    let out = &mut this_level_and_up[0];

    let mut total = 0u128;
    for (dice, count) in ALL_21 {
        let mult = if weighted { count as u128 } else { 1 };
        position.possible_positions_in(&dice, out);
        for next in out.iter() {
            total += mult * perft_all_rolls(next, depth - 1, weighted, lower_levels);
        }
    }
    total
}

fn usage() {
    println!("Usage: bkgm-perft [OPTIONS]");
    println!("  --position-id <ID>   GNUbg position ID (default: start)");
    println!("  --depth <N>          Perft depth (default: 2)");
    println!("  --iterations <N>     Number of repeated runs (default: 10)");
    println!("  --weighted           Use 36-roll weighting");
    println!("  --die0 <N> --die1 <N>  Single-roll benchmark mode");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if has_flag(&args, "--help") || has_flag(&args, "-h") {
        usage();
        return;
    }

    let position_id = args
        .windows(2)
        .find(|w| w[0] == "--position-id" || w[0] == "-p")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "4HPwATDgc/ABMA".to_string());

    let depth = parse_usize_flag(&args, "--depth")
        .or_else(|| parse_usize_flag(&args, "-d"))
        .unwrap_or(2);

    let iterations = parse_usize_flag(&args, "--iterations")
        .or_else(|| parse_usize_flag(&args, "-n"))
        .unwrap_or(10)
        .max(1);

    let weighted = has_flag(&args, "--weighted") || has_flag(&args, "-w");

    let die0 = parse_usize_flag(&args, "--die0");
    let die1 = parse_usize_flag(&args, "--die1");

    let position =
        <Position<15> as State>::from_id(position_id.as_str()).expect("invalid --position-id");

    if let (Some(d0), Some(d1)) = (die0, die1) {
        assert!(
            (1..=6).contains(&d0) && (1..=6).contains(&d1),
            "dice must be in 1..=6"
        );
        let dice = Dice::new(d0, d1);
        let started = Instant::now();
        let mut total_moves = 0usize;
        let mut out = Vec::with_capacity(256);

        for _ in 0..iterations {
            position.possible_positions_in(&dice, &mut out);
            total_moves += out.len();
        }

        let secs = started.elapsed().as_secs_f64();
        let it_per_sec = iterations as f64 / secs.max(1e-9);
        println!(
            "mode=single-roll position_id={} dice={},{} iterations={}",
            position_id, d0, d1, iterations
        );
        println!(
            "total_moves={} avg_moves={:.3}",
            total_moves,
            total_moves as f64 / iterations as f64
        );
        println!("time_s={:.6} it_per_sec={:.2}", secs, it_per_sec);
        return;
    }

    let started = Instant::now();
    let mut nodes = 0u128;
    let mut scratch: Vec<Vec<Position<15>>> =
        (0..=depth).map(|_| Vec::with_capacity(256)).collect();
    for _ in 0..iterations {
        nodes = perft_all_rolls(&position, depth, weighted, &mut scratch);
    }
    let secs = started.elapsed().as_secs_f64();
    let nps = (nodes as f64 * iterations as f64) / secs.max(1e-9);

    println!(
        "mode=perft position_id={} depth={} weighted={} iterations={}",
        position_id,
        depth,
        if weighted { 1 } else { 0 },
        iterations
    );
    println!("nodes_per_run={}", nodes);
    println!("time_s={:.6} nodes_per_sec={:.2}", secs, nps);
}
