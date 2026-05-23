# Assets

This directory is reserved for non-code project assets.

## Layout

```text
assets/
  prototype/
    third_party/
```

## Rules

- Put temporary external prototype art under `assets/prototype/third_party/`.
- Add a `NOTICE.md` in each imported source directory.
- Do not make `mundus_core` depend on asset files.
- Replace prototype third-party assets before any commercial release unless their
  provenance and shipping rights are explicitly confirmed.
