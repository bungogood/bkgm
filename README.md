# bkgm

[![Build](../../actions/workflows/build.yaml/badge.svg)](../../actions/workflows/build.yaml)

`bkgm` is an engine-first Rust core for backgammon and related variants.

The design goal is similar to `cozy-chess` in chess: keep the core fast and reusable, while exposing a practical API for bots, duellers, analysis tools, and experiments.

## What You Get

- Fast board representation with checker-count generics (`Position<N>`).
- Variant layer (`Variant`, `VariantSpec`, `VariantPosition`, `Game`).
- Rules-driven move generation via `PositionRules` / `VariantRules`.
- Built-in rulesets:
  - `ClassicRules` (current default behavior)
  - `NoHitRules` (example alternate rules profile)
- Position encoding support:
  - GNUbg Position ID
  - XGID board-part conversion (for all supported checker counts)
  - XGID full string parse/format struct (`Xgid`)

## API Shape

### 1) Position (board data)

- `Position<N>` is the board state primitive.
- `N` is checker count (`2`, `3`, `4`, `5`, `15`, ...).

### 2) Rules (move generation policy)

- Move generation is exposed through rules interfaces, not raw position methods:
  - `PositionRules<const N: u8>`
  - `VariantRules`

### 3) Game (orchestration)

- `Game` stores variant + current position and offers legal/apply helpers.
- `Game::play_episode_with::<R, _, _>(...)` supports policy-driven rollout loops.

## Quick Start

### Variant/Game usage

```rust
use bkgm::{Dice, Game, Variant};

let mut game = Game::new(Variant::Backgammon);
let legal = game.legal_positions(&Dice::new(3, 1));
game.set_position(legal[0]).unwrap();
```

### Explicit rules usage

```rust
use bkgm::{ClassicRules, Dice, PositionRules};
use bkgm::variants::BACKGAMMON;

let legal = <ClassicRules as PositionRules<15>>::legal_positions(BACKGAMMON, &Dice::new(3, 1));
```

### Alternate rules profile (`NoHitRules`)

```rust
use bkgm::{Dice, NoHitRules, PositionRules};
use bkgm::variants::BACKGAMMON;

let legal = <NoHitRules as PositionRules<15>>::legal_positions(BACKGAMMON, &Dice::new(6, 1));
```

## XGID and GNUbg IDs

- GNUbg Position ID is supported directly by `State::position_id()` / `State::from_id(...)`.
- XGID board part:
  - `Position::<N>::to_xgid_board()`
  - `Position::<N>::from_xgid_board(...)`
- Full XGID struct parse/format:
  - `Xgid::parse(...)`
  - `Xgid::format()`

Variant helpers:

- `Variant::from_xgid_board(...)`
- `VariantPosition::xgid_board()`

## Perft / Bench Utilities

`bkgm-perft` supports variants and defaults to `iterations=1`:

```bash
cargo run --release --bin bkgm-perft -- --variant backgammon --depth 4
cargo run --release --bin bkgm-perft -- --variant hypergammon --depth 3 --weighted
```

`corpus-bench` benchmarks corpus movegen throughput.

## Variant Coverage

Built-in variants include:

- Backgammon
- Nackgammon
- Longgammon
- Hypergammon (`2`, `3`, `4`, `5` checkers)

## Project Direction

- Keep correctness and speed first.
- Keep rules explicit in public move generation APIs.
- Expand format interoperability (XGID/GNUbg and match/session context) safely.

## References

- [docs.rs/bkgm](https://docs.rs/bkgm/latest/bkgm)
- [Wildbg](https://github.com/carsten-wenderdel/wildbg)
- [GNU Backgammon](https://www.gnu.org/software/gnubg/)
- [Enumerating Backgammon Positions: The Perfect Hash (1997)](https://api.semanticscholar.org/CorpusID:60574812)
