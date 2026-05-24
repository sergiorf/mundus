# Earth Pipeline

This directory owns the offline workflow for turning real-world Earth datasets
into Mundus assets.

The runtime should load local baked assets. It should not call public map tile
servers during play.

## Purpose

The pipeline has two outputs:

- a coarse simulation Earth asset for gameplay
- one or more visual tile pyramids for rendering

The inputs come from official public datasets documented in
[manifest.toml](manifest.toml).

## Directory layout

```text
tools/earth_pipeline/
  README.md
  manifest.toml
  fetch.sh
  cache/
    natural_earth/
    blue_marble/
    gebco/
    srtm/

assets/earth/
  README.md
  raw/
  sim/
  render/
```

`cache/` is for downloaded upstream archives and should stay out of git.

`assets/earth/raw/` is for normalized intermediate data and should stay out of
git unless a later decision explicitly changes that rule.

`assets/earth/sim/` will hold baked simulation assets.

`assets/earth/render/` will hold baked render LOD assets.

## Commands

The Rust scaffold currently provides two commands:

```bash
cargo run -p earth_pipeline -- verify
cargo run -p earth_pipeline -- scaffold-first-milestone
cargo run -p earth_pipeline -- bake-first-milestone
```

`verify` checks the first-milestone inputs listed in `manifest.toml` and reports
which matching files were found in the cache.

`scaffold-first-milestone` verifies the cached Natural Earth and Blue Marble
inputs, creates the initial output directories, and writes a first-milestone
target file at:

- `assets/earth/raw/first_milestone/targets.toml`

This is still a scaffold. It does not rasterize polygons or cut image tiles yet.

`bake-first-milestone` is the first real bake step. It:

- extracts `ne_110m_land.*` from the Natural Earth zip
- rasterizes a `360x180` Earth landmask into `assets/earth/sim/earth_360x180_landmask.toml`
- resizes Blue Marble into a `512x256` LOD0 atlas
- writes two `256x256` LOD0 tiles and a tile manifest under
  `assets/earth/render/lod0/`

## What you need to download now

Download these first:

1. Natural Earth physical vectors
2. NASA Blue Marble Next Generation imagery

These are enough to start:

- replacing the handcrafted Earth land shape
- baking a real Earth land mask
- building the first visual Earth layer

## What you can download later

Download these when you start elevation and relief work:

1. GEBCO global grid
2. SRTM 1 Arc-Second Global

For the first elevation pass, GEBCO is easier because it is global and already
packaged for whole-Earth use. SRTM should be added later for higher-quality land
elevation where available.

## Recommended first downloads

### 1. Natural Earth

Official downloads page:

- https://www.naturalearthdata.com/downloads/

Recommended first assets:

- `1:110m Physical Vectors`
- `1:50m Physical Vectors`
- `1:10m Physical Vectors`

Within those, the minimum useful themes are:

- land
- ocean
- coastline
- rivers and lake centerlines
- lakes

Why:

- `110m` is good for low zoom
- `50m` is good for medium zoom
- `10m` is good for close zoom

### 2. NASA Blue Marble

Official collection pages:

- https://science.nasa.gov/earth/earth-observatory/collections/blue-marble/
- https://science.nasa.gov/earth/earth-observatory/blue-marble-next-generation-5935/
- https://visibleearth.nasa.gov/images/5935/blue-marble-next-generation

Recommended first asset:

- one global true-color Blue Marble Next Generation image set suitable for
  raster tile generation

Use it for:

- low and medium zoom base color
- a realistic-looking Earth before terrain relief is added

### 3. GEBCO

Official page:

- https://www.gebco.net/data_and_products/gridded_bathymetry_data/

Recommended first asset:

- the current global GEBCO grid in tiled GeoTIFF or netCDF form

Use it for:

- whole-world elevation fallback
- ocean bathymetry
- early relief shading

### 4. SRTM

Official page:

- https://www.usgs.gov/centers/eros/science/usgs-eros-archive-digital-elevation-shuttle-radar-topography-mission-srtm-1-arc

Notes:

- access is through USGS tooling such as EarthExplorer
- downloads are heavier and more operationally awkward than the first two data
  sources

Use it for:

- higher-quality land elevation
- later refinement of mountain regions

## Download order

Do the downloads in this order:

1. Natural Earth
2. Blue Marble
3. GEBCO
4. SRTM

That order matches the implementation plan:

- phase 4 needs Natural Earth first
- phase 5 benefits from Blue Marble
- phase 6 needs GEBCO and optionally SRTM

## Where to put the files

Place downloaded upstream archives here:

- `tools/earth_pipeline/cache/natural_earth/`
- `tools/earth_pipeline/cache/blue_marble/`
- `tools/earth_pipeline/cache/gebco/`
- `tools/earth_pipeline/cache/srtm/`

Do not unpack directly into `crates/` or mix raw upstream files with runtime
assets.

## How to use the manifest

`manifest.toml` is the source-of-truth inventory for:

- what upstream datasets we depend on
- which ones are required for the current implementation phase
- where they should be stored locally
- what each dataset is used for

If you add a new Earth source, add it to the manifest first.

## Initial workflow

For now the workflow is:

1. run `tools/earth_pipeline/fetch.sh`
2. create the cache directories
3. read the printed instructions
4. manually download the required upstream files into the matching cache folders

The fetch script is intentionally conservative in this first version. Some
providers change direct links, and some downloads require navigation or account
steps. The repo should document the official source pages before it automates
network fetches.

## First milestone this pipeline should support

The first concrete output should be:

- a baked Earth land mask from Natural Earth for the simulation Earth preset

The second output should be:

- one Earth render LOD from Blue Marble imagery

Do not start with SRTM or road overlays. Start with land shape and one visual
layer.
