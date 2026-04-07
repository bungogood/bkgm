# bkgm Performance Log

## Goal

Improve move generation and perft throughput in `bkgm` using benchmark-driven changes.

## Benchmarks Used

- Perft baseline: start position `4HPwATDgc/ABMA`, depth 3, unweighted.
- Isolated perft binary: `src/bin/bkgm-perft.rs`.
- Movegen Criterion suite: `benches/movegen_bench.rs`.
- Corpus benchmark binary: `src/bin/corpus-bench.rs`.

## Kept Changes

1. Added reusable output-buffer API:
   - `Position::possible_positions_in(&self, dice, out: &mut Vec<Self>)`
   - `possible_positions()` now delegates to it.

2. Refactored movegen internals to write into caller-provided buffers:
   - `src/position/double_moves.rs`
   - `src/position/mixed_moves.rs`

3. Reused scratch vectors across perft recursion levels in `bkgm-perft`.

4. Increased hot-path move buffer capacities from `128` to `256`.

5. Added benchmark tooling and allocator alignment (`mimalloc`) in perf binaries.

## Measured Impact

- Depth-3 perft:
  - before: `~1.959s`
  - after: `~1.646s`
  - improvement: `~16%`

- Start single-roll throughput (3-1, long-run sampling):
  - before: `~4.40M it/s`
  - after: `~6.13M it/s`
  - improvement: `~43%`

- Corpus benchmark after allocator alignment:
  - small but repeatable gain in representative runs (`~2-3%` range), with notable run-to-run variance.

## Tested and Reverted

- Loop-style rewrite of hot iterator chains (regressed on this machine).
- Mixed-move pruning guard port from wildbg (regressed/changed outcomes in this codebase context).
- Inlining-only helper pass (no reliable win).

## Current Status

- Movegen/perft are materially faster than baseline.
- Bench discipline is in place (Criterion + corpus + isolated perft).
- Still optimizing toward parity or better against wildbg.

## Next Targets

1. Try `SmallVec`/stack-backed move buffers in the hottest paths.
2. Explore no-flip traversal mode for perft/search to reduce copy work.
3. Continue contact-position focused tuning with strict benchmark gates.
