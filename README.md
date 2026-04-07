# bkgm

[![Build](../../actions/workflows/build.yaml/badge.svg)](../../actions/workflows/build.yaml)

Bkgm is an engine-first Rust core for backgammon variants. It focuses on fast, correct move generation while still providing a user-friendly variant/game API for building bots, tools, services, and experiments.

The project direction is similar to the role of `cozy-chess` in chess: a reusable high-performance core that other projects can build on top of.

## API Layers

- Core hot path: `Position<N>` and movegen APIs (`possible_positions`, `possible_positions_in`).
- Variant UX layer: `Variant`, `VariantSpec`, `VariantPosition`, and `Game`.
- Presets: starting positions and built-in variants in `variants`.

Most consumers should start with the variant/game layer and use `Position<N>` directly when they need maximum control.

## Quick Start (Variant/Game UX)

```rust
use bkgm::{Dice, Game, Variant};

let mut game = Game::new(Variant::Backgammon);
let legal = game.legal_positions(&Dice::new(3, 1));
game.set_position(legal[0]).unwrap();
```

## Quick Start (Core API)

```rust
use bkgm::{Dice, State};
use bkgm::variants::BACKGAMMON;

let mut out = Vec::with_capacity(256);
BACKGAMMON.possible_positions_in(&Dice::new(3, 1), &mut out);
```

## Example Position

Here's a visual representation of the starting position in Backgammon:

```plaintext
Position ID: 4HPwATDgc/ABMA
┌13─14─15─16─17─18─┬───┬19─20─21─22─23─24─┬───┐
│ X           O    │   │ O              X │   │
│ X           O    │   │ O              X │   │
│ X           O    │   │ O                │   │
│ X                │   │ O                │   │
│ X                │   │ O                │   │
│                  │BAR│                  │OFF│
│ O                │   │ X                │   │
│ O                │   │ X                │   │
│ O           X    │   │ X                │   │
│ O           X    │   │ X              O │   │
│ O           X    │   │ X              O │   │
└12─11─10──9──8──7─┴───┴─6──5──4──3──2──1─┴───┘
```

## Features

- Support for GNUbg position IDs.
- Fast legal move/position generation.
- Variant presets (Backgammon, Nackgammon, Longgammon, Hypergammon variants).
- High-level game/variant API plus low-level core API.

## Performance Direction

- Move generation correctness and speed are the top priority.
- Benchmark-driven optimization is preferred over speculative refactors.
- We actively compare against strong references such as [Wildbg](https://github.com/carsten-wenderdel/wildbg).

## TODO

- Move parsing/notation (e.g., `24/23*/22*/21*`).
- Expand rule-profile support for more variant families.
- Improve test coverage and performance regression guardrails.

## References

-   Bkgm [Documentation](https://docs.rs/bkgm/latest/bkgm)
-   Backgammon [Wikipedia](https://en.wikipedia.org/wiki/Backgammon)
-   [Wildbg](https://github.com/carsten-wenderdel/wildbg) by [Carsten Wenderdel](https://github.com/carsten-wenderdel/wildbg)
-   [Enumerating Backgammon Positions: The Perfect Hash (1997)](https://api.semanticscholar.org/CorpusID:60574812) by A. T. Benjamin and A. M. Ross
