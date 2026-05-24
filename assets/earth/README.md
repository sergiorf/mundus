# Earth Assets

This directory is reserved for Earth-specific runtime and intermediate assets.

## Layout

```text
assets/earth/
  README.md
  raw/
  sim/
  render/
```

## Meaning

- `raw/` holds normalized intermediate files produced by the Earth pipeline
- `sim/` holds baked simulation Earth assets
- `render/` holds baked visual Earth tiles grouped by LOD

These directories are ignored by git by default because Earth source data and
derived tile sets can be large.

## Rules

- Do not put raw upstream archives here. Store them under
  `tools/earth_pipeline/cache/`.
- Do not make runtime code depend on GIS source formats directly.
- Add new datasets to `tools/earth_pipeline/manifest.toml` before using them.
- Prefer reproducible baked outputs over ad hoc manually edited files.
