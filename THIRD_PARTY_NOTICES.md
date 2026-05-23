# Third-Party Notices

This repository is licensed under the MIT License. Third-party code, art, audio,
or data used for prototypes must remain isolated and documented here.

## Current status

No third-party prototype assets are currently checked into this repository.

## Prototype asset policy

If temporary third-party assets are added for internal prototyping, they must:

1. live under `assets/prototype/third_party/`
2. include a source notice in the asset directory
3. preserve upstream copyright and license notices
4. be treated as temporary placeholders
5. be replaced before any commercial release unless provenance and licensing are
   confirmed for shipped use

## Intended import path

Temporary imports from external projects should use a structure such as:

```text
assets/prototype/third_party/<source-project>/
```

Each source-project directory should contain a `NOTICE.md` describing:

- upstream project name
- upstream repository URL
- upstream license claim
- imported files
- date imported
- removal/replacement requirement

## Current imports

None.
