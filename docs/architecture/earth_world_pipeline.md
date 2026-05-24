# Earth World Pipeline Plan

## Purpose

This document defines how Mundus should move from the current handcrafted Earth preset to a real multi-resolution Earth pipeline.

The goal is not only to make the world "look more like Earth". The goal is to support three things at the same time:

- real Earth land shapes
- much higher visual resolution
- progressive detail as the player zooms in

The current `world.rs` implementation does not have the right shape for that. It mixes simulation concerns and map generation concerns into one runtime step. That is acceptable for a prototype world generator, but it is the wrong foundation for a streamed Earth representation.

This plan replaces that with a clear split between:

- simulation data used by gameplay
- render data used by the viewer
- offline build steps used to transform real-world datasets into game assets

## Non-goals

This plan does not attempt to do all of the following in the first pass:

- globe projection correctness at every zoom level
- full GIS feature parity
- road-by-road or building-by-building Earth rendering
- real-time downloads from external map providers
- changing gameplay resolution every time the camera zooms

Those are either later optimizations or explicit anti-goals for the initial system.

## Core idea

We need two different representations of the world.

### Simulation world

The simulation world is the authoritative gameplay map.

It should:

- remain deterministic
- remain relatively coarse
- remain cheap to store and test
- drive movement, economy, ownership, and combat

It should not:

- try to store every visual detail visible at high zoom
- depend on image tiles or viewer state

### Render world

The render world is the visual representation shown by the viewer.

It should:

- support multiple levels of detail
- stream only visible data
- become richer as the player zooms in
- be allowed to contain more information than the simulation grid

It should not:

- become the source of truth for game rules
- force gameplay code to operate at image-tile resolution

### Offline asset pipeline

Real-world Earth data should be transformed ahead of time into local assets.

That pipeline should:

- download approved source datasets
- normalize them into the formats Mundus needs
- generate simulation-ready Earth data
- generate render-ready tiles for multiple LODs

The game should load local assets. It should not call third-party map services during play.

## Why this split matters

If we bind zoom level directly to gameplay resolution, three problems appear immediately:

- movement and ownership become unstable as visual detail changes
- memory and CPU use grow too quickly
- deterministic simulation becomes harder to reason about and test

By splitting the problem, we get:

- stable gameplay rules
- scalable rendering
- clean testing boundaries
- room to add richer visual Earth data later

## Data sources

The recommended input datasets are:

- Natural Earth for land polygons, coastlines, rivers, and broad cultural layers
- NASA Blue Marble for global base imagery
- SRTM for land elevation where available
- GEBCO for global terrain and ocean depth fallback
- OpenStreetMap-derived extracts only for later close-zoom overlays

Use these sources as offline inputs. Do not use public tile servers as a runtime dependency.

The repo-local source inventory and download workflow live in:

- [tools/earth_pipeline/README.md](/home/sergio/dev/mundus/tools/earth_pipeline/README.md)
- [tools/earth_pipeline/manifest.toml](/home/sergio/dev/mundus/tools/earth_pipeline/manifest.toml)
- [assets/earth/README.md](/home/sergio/dev/mundus/assets/earth/README.md)

## Architecture target

The target architecture should have three layers.

### Layer 1: Simulation

Owned by `mundus_core`.

Suggested responsibilities:

- world dimensions
- terrain and biome categories
- passability
- movement costs
- yields
- city placement rules
- deterministic game logic

Suggested types:

```rust
pub struct SimWorld {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<SimTile>,
}

pub struct SimTile {
    pub terrain: TerrainType,
    pub biome: Biome,
    pub elevation_band: u8,
    pub moisture_band: u8,
}
```

### Layer 2: Render

Owned by `mundus_core` as data structures and by `mundus_cli` as viewer behavior.

Suggested responsibilities:

- tile addressing
- tile loading
- LOD selection
- asset cache
- rendering color layers
- rendering terrain relief
- rendering high-zoom overlays

Suggested types:

```rust
pub struct RenderTileId {
    pub lod: u8,
    pub x: u32,
    pub y: u32,
}

pub struct RenderTile {
    pub id: RenderTileId,
    pub width: u16,
    pub height: u16,
    pub color: Vec<u8>,
    pub height: Option<Vec<u16>>,
    pub mask: Option<Vec<u8>>,
}
```

### Layer 3: Asset pipeline

Owned by a new tools directory or helper crate.

Suggested responsibilities:

- fetching source data
- reprojection and normalization
- rasterization
- simplification by LOD
- tile packaging
- simulation-grid baking

## Proposed file layout

This is the intended end-state layout, not a requirement for phase 1.

```text
docs/
  architecture/
    earth_world_pipeline.md

tools/
  earth_pipeline/
    README.md
    fetch.sh
    manifest.toml
    src/
      main.rs
      sources.rs
      rasterize.rs
      sim_bake.rs
      render_bake.rs

assets/
  earth/
    manifest.json
    sim/
      earth_360x180.bin
    render/
      lod0/
      lod1/
      lod2/
      lod3/

crates/
  mundus_core/
    src/
      world.rs
      world_sim.rs
      world_render.rs
      geo.rs
      lod.rs
      tile_pyramid.rs
      biome.rs
```

## Implementation phases

Each phase should leave the repo in a working state.

### Phase 0: Freeze the current prototype baseline

Purpose:

- keep the current Earth preset working
- avoid breaking the viewer while the new architecture is introduced

What to do:

- keep the current `WorldPreset::Earth` path available
- do not remove the procedural world path
- document the current limitations in code comments or issue tracking

Done when:

- `cargo test` still passes
- the viewer still launches
- no architecture work has broken the current prototype loop

### Phase 1: Split simulation and rendering responsibilities

Purpose:

- stop treating a single `World` struct as both simulation storage and visual map source

What to implement:

- create `world_sim.rs`
- create `world_render.rs`
- create `geo.rs`
- create `lod.rs`
- create `tile_pyramid.rs`
- create `biome.rs`

Recommended design:

- `world_sim.rs` owns gameplay map concepts
- `world_render.rs` owns render tile concepts
- `geo.rs` owns coordinate transforms
- `lod.rs` owns camera zoom to LOD selection logic
- `tile_pyramid.rs` owns tile addressing and bounds math

What you need to decide:

- whether the simulation grid stays equirectangular
- whether render tiles will use the same global coordinate basis in phase 1

Recommended answer:

- yes, keep the simulation grid equirectangular for now
- yes, keep render tiles on the same logical basis in phase 1

Why:

- it keeps the first implementation simpler
- it avoids introducing projection complexity before the asset pipeline exists

Done when:

- the code compiles with the new modules present
- the viewer still runs
- no visual Earth assets are required yet

### Phase 2: Define the simulation data model

Purpose:

- create a gameplay-oriented Earth representation that is richer than raw `TerrainType`

What to implement:

- add a `Biome` enum
- add `SimTile`
- add `SimWorld`
- define how Earth source data maps to gameplay terrain categories

Questions to answer in code:

- how many terrain classes should gameplay understand
- whether mountains are impassable or costly
- whether coastline is a terrain type or a render-only distinction
- whether `River` stays a terrain type or becomes a feature overlay

Recommended first answer:

- keep `TerrainType` for gameplay simplicity
- add `Biome` for realism and later yield tuning
- keep coast as render-only for now
- keep `River` as a gameplay terrain type until the system grows more sophisticated

Done when:

- simulation tiles can store both gameplay terrain and Earth-derived biome information
- current game rules still function

### Phase 3: Build the offline Earth pipeline skeleton

Purpose:

- establish the workflow that turns source datasets into local assets

What to implement:

- create `tools/earth_pipeline/README.md`
- create a manifest describing sources and outputs
- create fetch commands or scripts
- create a Rust entry point or script entry point for the bake process

The first version does not need to perform full rasterization. It only needs to define the pipeline shape and output locations.

Suggested commands:

- `fetch`: download source data into a local cache
- `bake-sim`: produce a coarse simulation Earth asset
- `bake-render`: produce one render LOD
- `verify`: check that outputs exist and basic metadata is valid

Done when:

- there is a repeatable pipeline entry point
- the expected output directory structure is defined
- the repo has a clear place for Earth assets

### Phase 4: Replace handcrafted Earth land with baked land data

Purpose:

- stop generating Earth land shapes from hardcoded ellipses

What to implement:

- bake a coarse Earth land mask from Natural Earth
- load that mask into the Earth preset instead of the current handcrafted model
- keep the simulation grid coarse, for example `360x180` or `720x360`

How it should work:

- source polygon data is rasterized offline
- the runtime loads a compact baked file
- `WorldPreset::Earth` builds `SimWorld` from baked data

Important rule:

- runtime code should not parse large GIS files directly

Why:

- runtime loading should stay fast and deterministic
- GIS tooling belongs in the pipeline, not the game loop

Done when:

- Earth landmasses come from baked real-world data
- the viewer no longer depends on hardcoded continent approximations

### Phase 5: Introduce one visual Earth LOD

Purpose:

- decouple what the player sees from the coarse simulation grid

What to implement:

- define one render tile format
- generate one set of color tiles from Blue Marble or equivalent imagery
- implement a local tile loader
- render Earth from visual tiles in the viewer

The first pass should be intentionally small:

- one LOD only
- fixed tile size such as `256x256`
- local loading only

What the viewer does in this phase:

- decide which Earth render tiles are visible
- load them from disk
- cache them
- draw them behind or instead of the coarse tile colors

Done when:

- the Earth preset can be drawn from tile assets
- the simulation world still exists independently underneath

### Phase 6: Add elevation and derived terrain relief

Purpose:

- make Earth look like terrain, not only like a flat colored map

What to implement:

- bake height tiles from SRTM and GEBCO
- add optional height storage to render tiles
- derive shading or relief in the viewer
- derive simulation `elevation_band` data from the same source

What not to do yet:

- full 3D terrain mesh rendering
- physically accurate globe rendering

Recommended first pass:

- keep the map 2D
- use hillshade or simple relief tinting
- use elevation for biome and mountain classification

Done when:

- mountain systems and major relief are visible at runtime
- the simulation can classify terrain using real elevation data

### Phase 7: Add multiple LODs

Purpose:

- make more detail appear as the player zooms in

What to implement:

- multiple tile directories such as `lod0`, `lod1`, `lod2`, `lod3`
- camera zoom to LOD mapping
- parent fallback when a child tile is missing
- memory-bounded tile cache

How it should behave:

- zoomed out: broad continents and color regions
- medium zoom: improved coastlines, major rivers, stronger relief
- close zoom: finer coastlines, lakes, terrain texture, overlays

Recommended viewer behavior:

- choose a target LOD from zoom level
- request visible tiles at that LOD
- if a tile is unavailable, temporarily draw the parent tile

Done when:

- zooming changes the visible detail level
- performance remains acceptable
- tile loads are local and predictable

### Phase 8: Add higher-zoom semantic overlays

Purpose:

- expose more geographic detail without increasing gameplay complexity

Possible overlays:

- rivers
- lakes
- climate zones
- city markers
- roads
- borders

Recommended order:

1. rivers
2. lakes
3. city markers
4. roads and borders

Important boundary:

- overlays should remain render-oriented until there is a strong gameplay reason to promote them into simulation data

Done when:

- close zoom reveals additional recognizable structure beyond base imagery

### Phase 9: Revisit gameplay integration

Purpose:

- decide what real-Earth detail should affect rules

Potential integrations:

- biome-aware yields
- mountain movement penalties
- river bonuses
- coastal city advantages
- desert settlement penalties

This phase should happen after the render pipeline is stable. Otherwise, the team will mix gameplay tuning with asset-pipeline debugging.

Done when:

- Earth-derived data affects gameplay intentionally rather than incidentally

## What you actually have to do

The work is easier if you think about it in three tracks.

### Track A: Core architecture

You need to:

- split world responsibilities into simulation and rendering modules
- add biome and Earth metadata to simulation tiles
- preserve deterministic behavior

This is the code-structure track.

### Track B: Data pipeline

You need to:

- create a reproducible way to fetch source datasets
- bake Earth source data into local runtime assets
- version the output formats

This is the tooling track.

### Track C: Viewer integration

You need to:

- teach the viewer how to select visible Earth tiles
- load local assets on demand
- cache them
- switch LODs as the camera zoom changes

This is the runtime presentation track.

## Step-by-step execution order

This is the recommended implementation order for the repo.

1. Add `biome.rs`.
2. Add `world_sim.rs` and move gameplay-oriented world concepts there.
3. Add `world_render.rs`, `geo.rs`, `lod.rs`, and `tile_pyramid.rs`.
4. Keep `world.rs` as a temporary facade.
5. Create `tools/earth_pipeline/`.
6. Add a documented but minimal bake pipeline.
7. Replace handcrafted Earth land with baked land data.
8. Add one render LOD and a local tile loader.
9. Change `mundus_cli` to draw the Earth preset from render tiles.
10. Add elevation tiles.
11. Add more LODs.
12. Add higher-zoom overlays.
13. Revisit gameplay integration.

That order minimizes risk because each step builds on a stable lower layer.

## Acceptance criteria by milestone

### Milestone A: Architecture split

Accept when:

- modules exist
- tests pass
- existing viewer still works

### Milestone B: Baked Earth simulation

Accept when:

- Earth landmasses come from baked data
- deterministic Earth tests still pass
- no hardcoded continent approximation remains in the runtime Earth path

### Milestone C: Visual Earth tiles

Accept when:

- the viewer loads Earth visuals from local tiles
- the simulation grid remains separate
- one LOD renders correctly

### Milestone D: Progressive zoom detail

Accept when:

- at least three LODs exist
- zoom changes visible detail
- cache behavior is stable

### Milestone E: Earth-aware gameplay

Accept when:

- biome and terrain data influence gameplay deliberately
- balance changes are testable
- simulation remains deterministic

## Risks and how to avoid them

### Risk: one giant runtime texture

Avoid by:

- tiling from the start
- loading only visible tiles

### Risk: gameplay coupled to render detail

Avoid by:

- keeping `SimWorld` authoritative
- treating render tiles as presentation data

### Risk: direct dependency on public map servers

Avoid by:

- pre-baking local assets
- never treating external tile servers as part of runtime

### Risk: overcomplicated first implementation

Avoid by:

- shipping one baked land layer first
- then one visual LOD
- then elevation
- then more LODs

### Risk: projection complexity too early

Avoid by:

- using simple global coordinate assumptions in the first pass
- deferring advanced projection work until the tile pipeline is proven

## Testing strategy

We need tests at three levels.

### Unit tests

Add tests for:

- coordinate normalization
- LOD selection
- tile index math
- asset manifest parsing
- baked Earth deterministic loading

### Data validation tests

Add checks for:

- known land positions such as Europe, Africa, South America, Australia
- known water positions such as Atlantic and Pacific samples
- expected elevation-heavy regions such as Andes and Himalaya

### Runtime behavior tests

Check manually or with targeted integration coverage:

- viewer loads Earth without crashing
- zoom changes LOD
- missing higher-LOD tiles fall back cleanly
- cache memory stays bounded

## Open decisions

These decisions do not block phase 1, but they should be revisited as implementation starts.

- exact simulation grid resolution
- exact render tile format
- whether to use PNG, custom binary, or both
- whether render loading belongs entirely in `mundus_cli` or partly in `mundus_core`
- whether rivers remain terrain or become a separate feature layer

## Recommended next action

The next implementation step should be phase 1, not dataset ingestion.

Specifically:

1. add the new world/render/geo/LOD module skeletons
2. move current simulation concerns out of `world.rs`
3. keep `world.rs` as a facade while the codebase transitions

That gives the project the structure needed for every later Earth improvement.
