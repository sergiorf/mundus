# Mundus

Mundus is a premium turn-based grand strategy prototype focused on a deterministic simulation core. The long-term goal is a reusable Rust foundation for compact strategy games covering economy, population, trade, logistics, war, diplomacy, AI civilizations, and procedural worlds.

The current milestone is intentionally small: a deterministic core plus an early GUI terrain viewer and a CLI gameplay prototype.

## Current milestone

- Deterministic simulation in `mundus_core`
- GUI terrain viewer in `mundus_cli`
- Text interface in `mundus_cli --cli`
- One human civilization and one AI civilization
- Grid map, cities, units, economy, combat, scoring, and win/loss rules
- Unit tests for deterministic behavior and core rules

## Workspace layout

```text
mundus/
  Cargo.toml
  LICENSE
  README.md
  THIRD_PARTY_NOTICES.md
  crates/
    mundus_core/
    mundus_cli/
  docs/
    architecture/
    design/
  assets/
  tools/
```

## Build

```bash
cargo build
```

## Test

```bash
cargo test
```

## Run the Terrain Viewer

```bash
cargo run -p mundus_cli
```

Viewer controls:

- `Mouse wheel`: zoom
- `Right` or `Middle mouse drag`: pan
- `WASD` or arrow keys: pan
- `R`: regenerate the terrain with the next seed

## Run the CLI

```bash
cargo run -p mundus_cli -- --cli
```

## License

Mundus is licensed under MIT. See `LICENSE`.

Temporary third-party prototype assets, if added, must be isolated under
`assets/prototype/third_party/` and documented in `THIRD_PARTY_NOTICES.md`.
Those assets are placeholders only and must be reviewed or replaced before any
commercial release.

Useful commands in the CLI:

- `help`
- `map`
- `status`
- `cities`
- `units`
- `city <id>`
- `set-project <city_id> militia|granary|workshop`
- `move <unit_id> <x> <y>`
- `attack-unit <unit_id> <target_unit_id>`
- `attack-city <unit_id> <target_city_id>`
- `end`
- `quit`

## Next milestones

1. Improve AI decision quality and map pressure.
2. Add save/load plus deterministic replay fixtures.
3. Add more projects, city specialization, and a second unit line.
4. Expand world generation and logistics without breaking core determinism.

## Additional architecture docs

- [Earth World Pipeline Plan](docs/architecture/earth_world_pipeline.md)

## Earth pipeline scaffold

- [Earth Pipeline README](tools/earth_pipeline/README.md)
- [Earth Source Manifest](tools/earth_pipeline/manifest.toml)
- `cargo run -p earth_pipeline -- verify`
