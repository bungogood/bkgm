# bkgm

[![Build](../../actions/workflows/build.yaml/badge.svg)](../../actions/workflows/build.yaml)

Bkgm is a versatile Rust crate designed to facilitate Backgammon-related operations, such as move generation, position parsing, conversion between standards, and the implementation of the [perfect hash](https://api.semanticscholar.org/CorpusID:60574812) for bearoff and hypergammon databases. It supports both traditional Backgammon and 3-checker Hypergammon and provides the foundation for building Backgammon engines or APIs.

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

-   Support for the Gnubg position id format.
-   Ability to generate possible positions.
-   Macros to create Backgammon and Hypergammon positions.
-   State trait implemented for both Backgammon and Hypergammon.

## TODO

-   Move generation (next possible position generation is already implemented).
-   Move parsing (e.g., 24/23*/22*/21\*).
-   Improve test coverage.
-   Add a game trait, enabling Mat files.
-   Addition of optional rules

## References

-   Bkgm [Documentation](https://docs.rs/bkgm/latest/bkgm)
-   Backgammon [Wikipedia](https://en.wikipedia.org/wiki/Backgammon)
-   [Wildbg](https://github.com/carsten-wenderdel/wildbg) by [Carsten Wenderdel](https://github.com/carsten-wenderdel/wildbg)
-   [Enumerating Backgammon Positions: The Perfect Hash (1997)](https://api.semanticscholar.org/CorpusID:60574812) by A. T. Benjamin and A. M. Ross
