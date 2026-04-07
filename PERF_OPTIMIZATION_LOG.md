# bkgm Movegen/Perft Performance Log

## Scope

This log covers the isolated benchmarking work and optimization pass done for `bkgm` move generation/perft.

## Benchmark setup

- Position: `4HPwATDgc/ABMA` (starting position)
- Main workload: depth-3 unweighted perft
- Command:

```bash
./target/release/bkgm-perft --position-id 4HPwATDgc/ABMA --depth 3 --iterations 1
```

- Stable measurement command:

```bash
hyperfine --warmup 3 --runs 10 './target/release/bkgm-perft --position-id 4HPwATDgc/ABMA --depth 3 --iterations 1'
```

## Changes made

### 1) Added isolated benchmark binaries

- Added GNUbg-side isolated movegen/perft tool:
  - `../nodots-backgammon-gnubg/bgperft.c`
  - wired in `../nodots-backgammon-gnubg/Makefile.am`
- Added matching bkgm tool:
  - `src/bin/bkgm-perft.rs`

Why: to benchmark core move generation/perft without UI noise.

### 2) Added reusable output-buffer API in `bkgm`

- Added `possible_positions_in(&self, dice, out: &mut Vec<Self>)`
  - file: `src/position.rs`
- `possible_positions()` now delegates to the new method.

Why: avoid repeated Vec allocations in deep recursive traversal.

### 3) Refactored mixed/double movegen to write into provided buffers

- Double path:
  - `all_positions_after_double_move_into(...)`
  - file: `src/position/double_moves.rs`
- Mixed path:
  - `all_positions_after_mixed_move_into(...)`
  - `moves_with_0_checkers_on_bar_into(...)`
  - `moves_with_1_checker_on_bar_into(...)`
  - `moves_with_2_checkers_on_bar_into(...)`
  - `two_checker_moves_into(...)`
  - file: `src/position/mixed_moves.rs`

Why: reduce allocator pressure and improve cache behavior in hot loops.

### 4) Updated perft recursion to reuse scratch vectors per depth

- `bkgm-perft` now carries a `scratch: &mut [Vec<Position<15>>]` through recursion.
- file: `src/bin/bkgm-perft.rs`

Why: remove per-node temporary vector creation in recursive perft.

### 5) Avoided copying `Position` values in recursion

- `perft_all_rolls` now iterates child positions by reference and recurses using a split scratch slice (`split_at_mut(depth)`), so each level uses a dedicated buffer without copying child structs.
- file: `src/bin/bkgm-perft.rs`

Why: reduce per-node `Position` copy overhead in deep recursion.

### 6) Increased default move buffer capacities in hot paths

- Increased `MOVES_CAPACITY` from `128` to `256`.
  - file: `src/position.rs`
- Increased `bkgm-perft` scratch/output vector capacities to `256`.
  - file: `src/bin/bkgm-perft.rs`

Why: reduce occasional realloc/grow events in deeper perft traversal.

### 7) Added dedicated move-generation benchmark suite

- Added Criterion benches:
  - `benches/movegen_bench.rs`
- Added dev dependencies and bench config:
  - `Cargo.toml`
- Bench coverage:
  - `movegen/start/single_roll_31`
  - `movegen/contact/all21` (sampled contact positions)
  - `movegen/race/all21` (sampled race positions)

Why: track movegen performance independently from perft recursion details and guard against regressions by position class.

### 8) Enabled `mimalloc` in isolated benchmark binaries

- Added global allocator in:
  - `src/bin/bkgm-perft.rs`
  - `src/bin/corpus-bench.rs`
- Moved `mimalloc` to regular dependencies so bin targets can use it.
  - `Cargo.toml`

Why: align allocator strategy with comparable benchmark setups and reduce allocator overhead in long-running movegen/perft loops.

## Measured impact

### Depth-3 perft (primary metric)

- Before optimization (measured):
  - `1.959 s ± 0.020`
- After pass 1 (buffer reuse):
  - `1.723 s ± 0.024`
- After pass 2 (child-by-ref recursion):
  - `1.653 s ± 0.016`
- After pass 3 (capacity tuning):
  - `1.646 s ± 0.010`

Improvement:

- Absolute: `-0.306 s`
- Relative: **~16.0% faster**
- Throughput gain: from ~`59.0M` to ~`70.6M` nodes/s (hyperfine-equivalent).

### Single-roll throughput (start position, 3-1, 200000 iters)

- Before: ~`4.40M it/s`
- After pass 1: ~`5.54M it/s`
- After pass 2: ~`6.31M it/s`
- After pass 3: ~`6.13M it/s` (run-to-run variance observed)

Improvement:

- Relative: **~43% faster**

### Wildbg corpus harness (1000 positions, all 21 dice)

- Before allocator alignment (sample run):
  - contact: ~`40.37M` moves/s
  - race: ~`45.31M` moves/s
- After `mimalloc` in bin targets (sample run):
  - contact: ~`41.32M` moves/s
  - race: ~`46.54M` moves/s

Observed improvement:

- Relative: roughly **+2% to +3%** in representative runs.
- Note: this harness has measurable run-to-run variance (thermal/load effects), so compare medians across repeated runs.

## Notes

- `cargo test` remains green except for one pre-existing `from_id` inconsistency in a legacy test path unrelated to this optimization work.
- Flamegraph and samples indicated allocator activity as a dominant overhead; this pass specifically targeted that and produced measurable gains.
- A separate loop-style rewrite experiment (replacing iterator chains in movegen) was tested and reverted because it regressed depth-3 perft on this machine.
- A direct port of a wildbg mixed-move pruning guard was tested and reverted; it regressed start/contact benchmarks on this machine.
- Additional inlining-only pass on movegen helpers was tested and reverted (no clear win; slight regression/noise in contact-heavy runs).

## Next recommended experiments

1. Replace hot iterator chains in mixed movegen with plain `for` loops.
2. Add no-flip traversal mode for perft/search to avoid child flip copy in tight loops.
3. Prototype make/unmake traversal path for perft.
4. Consider fixed-capacity stack storage for common move counts (e.g. `SmallVec`) to further reduce heap traffic.
