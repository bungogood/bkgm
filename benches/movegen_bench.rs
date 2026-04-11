use bkgm::dice::ALL_21;
use bkgm::position::{GamePhase, OngoingPhase};
use bkgm::variants::BACKGAMMON;
use bkgm::{legal_positions_with, ClassicRules, Dice, Position, State};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const SAMPLE_POSITIONS: usize = 512;

fn count_all21_moves(positions: &[Position<15>]) -> usize {
    let mut total = 0usize;
    for pos in positions {
        for (dice, _) in ALL_21 {
            total += legal_positions_with::<ClassicRules, 15>(*pos, &dice).len();
        }
    }
    total
}

fn collect_phase_positions(phase: OngoingPhase, count: usize, seed: u64) -> Vec<Position<15>> {
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut out = Vec::with_capacity(count);

    while out.len() < count {
        let mut p = BACKGAMMON;
        for _ply in 0..80 {
            let (dice, _) = ALL_21[rng.usize(0..ALL_21.len())];
            let scratch = legal_positions_with::<ClassicRules, 15>(p, &dice);
            if scratch.is_empty() {
                break;
            }
            p = scratch[rng.usize(0..scratch.len())];

            if let GamePhase::Ongoing(current) = p.phase() {
                if current == phase {
                    out.push(p);
                    break;
                }
            }
        }
    }

    out
}

fn bench_start_single_roll(c: &mut Criterion) {
    let pos = BACKGAMMON;
    let dice = Dice::new(3, 1);

    c.bench_function("movegen/start/single_roll_31", |b| {
        b.iter(|| {
            let out = legal_positions_with::<ClassicRules, 15>(pos, black_box(&dice));
            black_box(out.len())
        })
    });
}

fn bench_contact_all21(c: &mut Criterion) {
    let positions = collect_phase_positions(OngoingPhase::Contact, SAMPLE_POSITIONS, 7);
    let total = count_all21_moves(&positions);
    println!(
        "{} sampled contact positions, 21 dice outcomes -> {} generated moves.",
        SAMPLE_POSITIONS, total
    );
    c.bench_function("movegen/contact/all21", |b| {
        b.iter(|| black_box(count_all21_moves(black_box(&positions))))
    });
}

fn bench_race_all21(c: &mut Criterion) {
    let positions = collect_phase_positions(OngoingPhase::Race, SAMPLE_POSITIONS, 11);
    let total = count_all21_moves(&positions);
    println!(
        "{} sampled race positions, 21 dice outcomes -> {} generated moves.",
        SAMPLE_POSITIONS, total
    );
    c.bench_function("movegen/race/all21", |b| {
        b.iter(|| black_box(count_all21_moves(black_box(&positions))))
    });
}

criterion_group!(
    benches,
    bench_start_single_roll,
    bench_contact_all21,
    bench_race_all21
);
criterion_main!(benches);
