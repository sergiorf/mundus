#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"

mkdir -p \
  "$repo_root/tools/earth_pipeline/cache/natural_earth" \
  "$repo_root/tools/earth_pipeline/cache/blue_marble" \
  "$repo_root/tools/earth_pipeline/cache/gebco" \
  "$repo_root/tools/earth_pipeline/cache/srtm" \
  "$repo_root/assets/earth/raw" \
  "$repo_root/assets/earth/sim" \
  "$repo_root/assets/earth/render"

cat <<'EOF'
Earth pipeline directories are ready.

Download these official datasets into the matching cache folders:

1. Natural Earth physical vectors
   Page:
   https://www.naturalearthdata.com/downloads/
   Put archives in:
   tools/earth_pipeline/cache/natural_earth/

2. NASA Blue Marble Next Generation
   Pages:
   https://science.nasa.gov/earth/earth-observatory/collections/blue-marble/
   https://visibleearth.nasa.gov/images/5935/blue-marble-next-generation
   Put downloads in:
   tools/earth_pipeline/cache/blue_marble/

Later downloads for relief work:

3. GEBCO global grid
   Page:
   https://www.gebco.net/data_and_products/gridded_bathymetry_data/
   Put downloads in:
   tools/earth_pipeline/cache/gebco/

4. USGS SRTM 1 Arc-Second Global
   Page:
   https://www.usgs.gov/centers/eros/science/usgs-eros-archive-digital-elevation-shuttle-radar-topography-mission-srtm-1-arc
   Put downloads in:
   tools/earth_pipeline/cache/srtm/

Read:
tools/earth_pipeline/README.md
tools/earth_pipeline/manifest.toml

This script does not fetch the files yet. The first version is documentation and
directory scaffolding only.
EOF
