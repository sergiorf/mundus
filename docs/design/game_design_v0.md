# Mundus Game Design v0

## Vision

Mundus aims to become a serious strategy simulation line with reusable systems for economy, population, logistics, warfare, diplomacy, and AI civilizations. Version 0 does not attempt to realize that full scope. Its job is to prove that the turn structure and simulation foundation are worth expanding.

## v0 scope

Version 0 includes:

- square grid world
- terrain-based yields
- one human civilization
- one AI civilization
- one capital each at start
- city population, food, production, gold, and knowledge
- simple city projects
- militia units
- deterministic combat
- score target by turn 80

## Core loop

Each round follows a compact structure:

1. The player inspects the world state.
2. The player changes city production, moves units, and attacks.
3. The player ends the turn.
4. Cities produce yields and update growth or decline.
5. AI takes its turn.
6. Score and win/loss state are recalculated.

The loop is intentionally small so iteration can focus on whether decisions matter.

## Economy model

Cities work nearby tiles. Terrain provides fixed base yields. Population consumes food every turn. Food surplus fills storage and can trigger growth. Food deficit drains storage and can reduce population. Production accumulates toward a current city project. Gold and knowledge are tracked globally per player.

Initial projects are:

- `Train Militia`
- `Build Granary`
- `Build Workshop`

These projects create a minimal tension between defense and economic development.

## War model

Units use movement points on a square grid and engage in deterministic combat. Cities have defense strength and hit points. If a capital is destroyed, its owner loses. The model is simple on purpose: it creates pressure without requiring a larger tactical ruleset yet.

## AI model

The AI is intentionally basic. It trains militia when weak, otherwise improves economy, moves toward enemy cities, attacks adjacent enemies, and ends its turn. The target is not sophistication. The target is to create pressure, expose balance problems, and exercise the simulation.

## Win and loss

The player wins by reaching the target score by turn 80 or by destroying the enemy capital first. The player loses if the capital is destroyed, total population collapses, or turn 80 is reached without enough score.

## Intentionally excluded from v0

The prototype excludes:

- multiplayer
- networking
- accounts
- cloud deployment
- graphics-heavy frontend
- procedural diplomacy systems
- trade networks
- logistics chains
- multiple unit classes
- technology tree
- full save/load UX

Those belong to later milestones after the core loop proves itself.
