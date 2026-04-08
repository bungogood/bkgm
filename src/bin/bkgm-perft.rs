use std::env;
use std::time::Instant;

use bkgm::dice::ALL_21;
use bkgm::{ClassicRules, Dice, Position, PositionRules, State, Variant};
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

fn perft_all_rolls_n<const N: u8>(position: &Position<N>, depth: usize, weighted: bool) -> u128 {
    if depth == 0 {
        return 1;
    }

    let mut total = 0u128;
    for (dice, count) in ALL_21 {
        let mult = if weighted { count as u128 } else { 1 };
        let legal = <ClassicRules as PositionRules<N>>::legal_positions(*position, &dice);
        for next in legal.iter() {
            total += mult * perft_all_rolls_n(next, depth - 1, weighted);
        }
    }
    total
}

fn usage() {
    println!("Usage: bkgm-perft [OPTIONS]");
    println!("  --position-id <ID>   GNUbg position ID (default: start)");
    println!("  --variant <NAME>     backgammon|nackgammon|longgammon|hypergammon|hypergammon2|hypergammon4|hypergammon5 (default: backgammon)");
    println!("  --depth <N>          Perft depth (default: 2)");
    println!("  --iterations <N>     Number of repeated runs (default: 1)");
    println!("  --weighted           Use 36-roll weighting");
    println!("  --die0 <N> --die1 <N>  Single-roll benchmark mode");
}

fn parse_variant_flag(args: &[String]) -> Variant {
    let raw = args
        .windows(2)
        .find(|w| w[0] == "--variant" || w[0] == "-v")
        .map(|w| w[1].to_ascii_lowercase())
        .unwrap_or_else(|| "backgammon".to_string());

    match raw.as_str() {
        "backgammon" | "bg" => Variant::Backgammon,
        "nackgammon" | "nack" => Variant::Nackgammon,
        "longgammon" | "long" => Variant::Longgammon,
        "hypergammon" | "hyper" | "hypergammon3" => Variant::Hypergammon,
        "hypergammon2" | "hyper2" => Variant::Hypergammon2,
        "hypergammon4" | "hyper4" => Variant::Hypergammon4,
        "hypergammon5" | "hyper5" => Variant::Hypergammon5,
        _ => panic!("invalid --variant '{}'", raw),
    }
}

fn default_position_id_for_variant(variant: Variant) -> String {
    variant.start_position().position_id()
}

fn run_single_roll<const N: u8>(
    position: Position<N>,
    variant: Variant,
    position_id: &str,
    d0: usize,
    d1: usize,
    iterations: usize,
) {
    assert!(
        (1..=6).contains(&d0) && (1..=6).contains(&d1),
        "dice must be in 1..=6"
    );
    let dice = Dice::new(d0, d1);
    let started = Instant::now();
    let mut total_moves = 0usize;

    for _ in 0..iterations {
        total_moves += <ClassicRules as PositionRules<N>>::legal_positions(position, &dice).len();
    }

    let secs = started.elapsed().as_secs_f64();
    let it_per_sec = iterations as f64 / secs.max(1e-9);
    println!(
        "mode=single-roll variant={:?} position_id={} dice={},{} iterations={}",
        variant, position_id, d0, d1, iterations
    );
    println!(
        "total_moves={} avg_moves={:.3}",
        total_moves,
        total_moves as f64 / iterations as f64
    );
    println!("time_s={:.6} it_per_sec={:.2}", secs, it_per_sec);
}

fn run_perft<const N: u8>(
    position: Position<N>,
    variant: Variant,
    position_id: &str,
    depth: usize,
    weighted: bool,
    iterations: usize,
) {
    let started = Instant::now();
    let mut nodes = 0u128;
    for _ in 0..iterations {
        nodes = perft_all_rolls_n(&position, depth, weighted);
    }
    let secs = started.elapsed().as_secs_f64();
    let nps = (nodes as f64 * iterations as f64) / secs.max(1e-9);

    println!(
        "mode=perft variant={:?} position_id={} depth={} weighted={} iterations={}",
        variant,
        position_id,
        depth,
        if weighted { 1 } else { 0 },
        iterations
    );
    println!("nodes_per_run={}", nodes);
    println!("time_s={:.6} nodes_per_sec={:.2}", secs, nps);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if has_flag(&args, "--help") || has_flag(&args, "-h") {
        usage();
        return;
    }

    let variant = parse_variant_flag(&args);

    let position_id = args
        .windows(2)
        .find(|w| w[0] == "--position-id" || w[0] == "-p")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| default_position_id_for_variant(variant));

    let depth = parse_usize_flag(&args, "--depth")
        .or_else(|| parse_usize_flag(&args, "-d"))
        .unwrap_or(2);

    let iterations = parse_usize_flag(&args, "--iterations")
        .or_else(|| parse_usize_flag(&args, "-n"))
        .unwrap_or(1)
        .max(1);

    let weighted = has_flag(&args, "--weighted") || has_flag(&args, "-w");

    let die0 = parse_usize_flag(&args, "--die0");
    let die1 = parse_usize_flag(&args, "--die1");

    match variant {
        Variant::Backgammon | Variant::Nackgammon | Variant::Longgammon => {
            let position = <Position<15> as State>::from_id(position_id.as_str())
                .expect("invalid --position-id for selected variant");
            if let (Some(d0), Some(d1)) = (die0, die1) {
                run_single_roll(position, variant, &position_id, d0, d1, iterations);
            } else {
                run_perft(position, variant, &position_id, depth, weighted, iterations);
            }
        }
        Variant::Hypergammon => {
            let position = <Position<3> as State>::from_id(position_id.as_str())
                .expect("invalid --position-id for selected variant");
            if let (Some(d0), Some(d1)) = (die0, die1) {
                run_single_roll(position, variant, &position_id, d0, d1, iterations);
            } else {
                run_perft(position, variant, &position_id, depth, weighted, iterations);
            }
        }
        Variant::Hypergammon2 => {
            let position = <Position<2> as State>::from_id(position_id.as_str())
                .expect("invalid --position-id for selected variant");
            if let (Some(d0), Some(d1)) = (die0, die1) {
                run_single_roll(position, variant, &position_id, d0, d1, iterations);
            } else {
                run_perft(position, variant, &position_id, depth, weighted, iterations);
            }
        }
        Variant::Hypergammon4 => {
            let position = <Position<4> as State>::from_id(position_id.as_str())
                .expect("invalid --position-id for selected variant");
            if let (Some(d0), Some(d1)) = (die0, die1) {
                run_single_roll(position, variant, &position_id, d0, d1, iterations);
            } else {
                run_perft(position, variant, &position_id, depth, weighted, iterations);
            }
        }
        Variant::Hypergammon5 => {
            let position = <Position<5> as State>::from_id(position_id.as_str())
                .expect("invalid --position-id for selected variant");
            if let (Some(d0), Some(d1)) = (die0, die1) {
                run_single_roll(position, variant, &position_id, d0, d1, iterations);
            } else {
                run_perft(position, variant, &position_id, depth, weighted, iterations);
            }
        }
    }
}
