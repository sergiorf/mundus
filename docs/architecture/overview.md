# Mundus Architecture Overview

## Why `mundus_core` is independent

`mundus_core` contains the deterministic simulation model and game rules. It owns the map, turns, cities, units, economy, combat, scoring, and AI logic. It does not depend on CLI, server, renderer, or platform code.

That separation matters because the simulation is the product risk in this milestone. If the turn loop is not interesting, no amount of frontend work will fix the project direction. A clean core also keeps tests fast and allows rules to be exercised without UI concerns.

## Deterministic simulation design

The simulation uses explicit domain types and a seeded internal RNG for map generation. Runtime rules do not depend on wall clock time, concurrency, or non-deterministic APIs. The same seed and the same sequence of actions produce the same `GameState`.

This makes balancing, testing, debugging, and future replay support much simpler. It also keeps AI behavior stable across runs while the design is still changing.

## Why CLI comes first

The CLI is the thinnest possible interface that can prove whether the simulation works. It is cheap to build, fast to iterate on, and exposes the core mechanics directly:

- inspect the map
- inspect cities and units
- choose city projects
- move and attack
- end turns and observe AI responses

That is enough to validate the prototype loop before investing in graphics, networking, or persistence layers.

## Future extension path

Because `mundus_core` is isolated, future clients can wrap the same logic:

- desktop UI
- web client
- mobile client
- authoritative server
- replay viewer

Those layers should translate user intent into `PlayerAction` values and render `GameState` plus `TurnReport` output. They should not own gameplay rules.

## Active architecture notes

- [Earth World Pipeline](./earth_world_pipeline.md)
- [Strategic Simulation Architecture](./strategic_simulation.md)

## Save/load and replay support

The core types derive `serde` traits so future save/load can serialize `GameState`, `GameConfig`, and action logs. Two low-risk extensions follow naturally:

1. Save snapshots of current state.
2. Record initial seed plus player actions for deterministic replay.

That approach supports debugging and balancing without redesigning the architecture later.
