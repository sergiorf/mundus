# Prototype Asset Policy

Mundus may use temporary third-party art during prototype stages to accelerate
debugging and interaction design. That is acceptable only if the assets are
isolated, documented, and easy to replace.

## Rules

1. `mundus_core` must never depend on asset files.
2. Prototype third-party assets must live under `assets/prototype/third_party/`.
3. Every external asset source must have a `NOTICE.md` next to the imported files.
4. Production or commercial builds must not silently ship placeholder art.
5. All UI code should resolve art through a small asset layer so replacement does
   not require gameplay code changes.

## Why this exists

Prototype art can help evaluate map readability, city placement, unit contrast,
and interaction affordances long before final art direction is ready. The risk is
that placeholder assets become sticky and leak into release builds. This policy
keeps that debt visible.

## Suggested workflow

1. Import only the minimum files needed for a prototype.
2. Record provenance immediately.
3. Keep filenames stable behind local aliases where possible.
4. Replace third-party prototype assets before any commercial milestone.

## Current direction

The repository currently favors first-party placeholder assets under
`assets/prototype/local/` for early world-map work. Third-party prototype assets
may still be imported later if needed, but they should remain optional and
isolated.
