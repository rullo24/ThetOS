# ADR-0011: External projects import ThetOS by git, linker scripts self-locate

Status: Accepted

## Context
Firmware built on ThetOS (the RC car, future applications) should live in its own repository and depend on ThetOS like any library — not be a member of this workspace. The blocker was the linker: `.cargo/config.toml` passed workspace-root-relative paths to `memory.x` and `common_minimal.ld`, which an external project cannot resolve.

## Decision
- `mcu/stm32/stm32l152ret6` and `arch/cortex-m` each carry a `build.rs` that copies their linker script into `OUT_DIR` and emits `cargo:rustc-link-search` (the `cortex-m-rt` pattern). Consumers link with bare `-Tmemory.x -Tcommon_minimal.ld`.
- Consumers depend on `nucleo_l152re` + `thetos_entry` by git ref and name the board explicitly: `#[entry(bsp = nucleo_l152re)]`. The `THETOS_BSP` env / `build.rs` injection stays for in-tree demos only.
- Consumers set `panic = "abort"` and the target triple in their own manifest / `.cargo/config.toml` (profile and build settings do not propagate from a dependency).

## Alternatives
- Publish to crates.io: not warranted for a thesis project; still needs the linker fix.
- A `thetos-build` helper crate the consumer calls from its own `build.rs`: more machinery than `#[entry(bsp = …)]`.

## Consequences
- One linker mechanism for in-tree and external builds.
- Validated: `ThetOS_RC_Demo` builds via `path` and `git` deps; ELF layout identical to the in-tree demo.
- Recipe: [docs/external-usage.md](../external-usage.md).
