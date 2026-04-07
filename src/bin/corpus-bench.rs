use std::fs;
use std::time::Instant;

use bkgm::dice::ALL_21;
use bkgm::{Position, State};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn load_positions(path: &str) -> Vec<Position<15>> {
    fs::read_to_string(path)
        .expect("could not read corpus file")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|id| <Position<15> as State>::from_id(&id.to_string()).expect("invalid position id"))
        .collect()
}

fn parse_string_flag(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == name).map(|w| w[1].clone())
}

fn parse_usize_flag(args: &[String], name: &str) -> Option<usize> {
    args.windows(2)
        .find(|w| w[0] == name)
        .and_then(|w| w[1].parse::<usize>().ok())
}

fn run_once(positions: &[Position<15>]) -> usize {
    let mut out = Vec::with_capacity(256);
    let mut total = 0usize;
    for pos in positions {
        for (dice, _) in ALL_21 {
            pos.possible_positions_in(&dice, &mut out);
            total += out.len();
        }
    }
    total
}

fn bench(name: &str, positions: &[Position<15>], iterations: usize) {
    let started = Instant::now();
    let mut total_moves = 0usize;
    for _ in 0..iterations {
        total_moves = run_once(positions);
    }
    let secs = started.elapsed().as_secs_f64();
    let it_per_sec = iterations as f64 / secs.max(1e-9);
    let moves_per_sec = (total_moves as f64 * iterations as f64) / secs.max(1e-9);
    println!("{}", name);
    println!("positions={} iterations={}", positions.len(), iterations);
    println!("moves_per_run={}", total_moves);
    println!("time_s={:.6}", secs);
    println!("iters_per_sec={:.2}", it_per_sec);
    println!("moves_per_sec={:.2}", moves_per_sec);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let contact_path = parse_string_flag(&args, "--contact-csv").unwrap_or_else(|| {
        panic!("missing --contact-csv <path>");
    });
    let race_path = parse_string_flag(&args, "--race-csv").unwrap_or_else(|| {
        panic!("missing --race-csv <path>");
    });
    let iterations = parse_usize_flag(&args, "--iterations").unwrap_or(20).max(1);

    let contact = load_positions(&contact_path);
    let race = load_positions(&race_path);

    bench("corpus/contact/all21", &contact, iterations);
    bench("corpus/race/all21", &race, iterations);
}
