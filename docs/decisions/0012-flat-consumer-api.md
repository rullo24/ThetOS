# ADR-0012: Flat consumer API — one macro crate, one board crate

Status: Accepted

## Context
Before this, an application imported from three crate names: `entry` (a generic, owner-less name), `nucleo_l152re`, and `specs` — the internal contracts crate, whose name a consumer should never need to type.

## Decision
- Rename the `entry` crate to `thetos_entry`.
- Each BSP crate re-exports the application-facing task and GPIO types it composes, so `specs` becomes an internal implementation detail. (The current list is in the BSP crate's `lib.rs`.)

Result: application code depends on exactly two crates — `thetos_entry` and the board crate — and the story is "depend on your board's BSP crate plus the entry macro."

## Alternatives
- A `thetos` facade crate with `thetos::prelude` and feature-selected board (embassy-style): nicer front door, but needs board-feature machinery and macro/feature interaction design. Deferred; for a thesis about modular composition, naming the BSP crate explicitly is also more instructive.

## Consequences
- Consumers never name `specs`, `kernel`, `arch`, or `mcu`.
- The re-export list in each BSP crate must track the app-facing surface as it grows.
- `thetos_entry` is unambiguous and greppable.
