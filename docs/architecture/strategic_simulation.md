# Strategic Simulation Architecture

This document tracks the current gameplay-side architecture for the world map.
It should be updated when the tile model, founding rules, city economy, or
viewer contracts change.

## Scope

This document covers:

- simulation tile data in `mundus_core`
- city founding suitability
- the contract between `mundus_core` and `mundus_cli`

It does not cover:

- render tile baking internals
- Earth source ingestion details
- future server or persistence architecture

## Current model

The world map is driven by simulation tiles, not by the rendered Earth image.

```mermaid
flowchart LR
    A[Earth Inputs / Runtime Heuristics] --> B[World Generation]
    B --> C[Tile]
    C --> D[TerrainType]
    C --> E[TileMetadata]
    E --> F[Biome]
    E --> G[Fertility / Moisture / Temperature / Elevation / Ruggedness / Coast]
    C --> H[Tile::base_yield]
    C --> I[Tile::is_passable]
```

## Tile contract

`Tile` is the gameplay-facing terrain object.

- `terrain` is the coarse rule category.
- `metadata` carries richer simulation signals.
- `Tile` methods expose gameplay contracts such as passability and base yield.

```mermaid
classDiagram
    class Tile {
      +TerrainType terrain
      +TileMetadata metadata
      +is_passable()
      +is_land()
      +base_yield()
    }

    class TileMetadata {
      +u8 elevation
      +u8 moisture
      +u8 temperature
      +u8 fertility
      +u8 ruggedness
      +u8 water_distance
      +bool coastal
      +Biome biome
    }
```

## Founding suitability

City founding uses a reusable site-scoring function in `mundus_core::site`.

The scoring function answers two questions:

1. Is the tile valid for founding?
2. If valid, how desirable is it?

```mermaid
flowchart TD
    A[Candidate Tile] --> B{In Bounds?}
    B -- no --> X[Invalid]
    B -- yes --> C{Land / Not Mountain?}
    C -- no --> X
    C -- yes --> D{No City Nearby?}
    D -- no --> X
    D -- yes --> E[Read Tile + Neighbor Metadata]
    E --> F[Food Score]
    E --> G[Production Score]
    E --> H[Trade Score]
    E --> I[Climate Score]
    E --> J[Space / Coast Bonus]
    F --> K[FoundingSiteScore]
    G --> K
    H --> K
    I --> K
    J --> K
```

## Separation of concerns

`mundus_core` owns:

- tile and metadata generation
- founding rules
- founding mutation from settler to city
- economy rules
- city and unit state
- AI use of gameplay scoring contracts

`mundus_cli` owns:

- camera and selection state
- map rendering
- viewer panels
- displaying tile/founding diagnostics
- overlay visualizations derived from core scores

```mermaid
flowchart LR
    A[mundus_core / GameState] --> B[PlayerAction]
    B --> A
    A --> C[FoundingSiteScore]
    A --> G[AI Settler Logic]
    A --> D[Tile / TileMetadata]
    D --> E[mondus_cli Viewer]
    C --> E
    C --> F[Founding Overlay]
    C --> G
```

## Current follow-up work

- tune and extend the desirability overlay in the viewer
- use metadata more aggressively in city economy and placement
- move stable Earth simulation metadata to offline bake once the schema settles
