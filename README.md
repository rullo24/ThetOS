# ThetOS — a modular real-time operating system in Rust

An investigation into compile-time safety invariants and zero-cost modularity for mechatronic systems.

## Thesis problem statement

> Current industry-standard RTOS architectures (typically C-based) lack the ability to enforce memory safety and hardware-state invariants at compile-time, resulting in runtime failures that are difficult to detect and debug. This thesis seeks to prove that a modular RTOS developed in Rust, utilising static dispatch and the typestate pattern, can eliminate these failure modes at the compilation stage without incurring a noticeable performance penalty compared to a C-based equivalent.

## Implementation constraints

Set to keep the safety claims meaningful:

- Standard architecture runtime crates may be used for vector-table and startup logic.
- Register definitions may come from a Peripheral Access Crate, but all typestate logic and trait implementations are original.
- High-level Hardware Abstraction Layers are not used.
- The kernel is hardware-blind: no architecture- or board-specific code in it.
- `unsafe` is confined to the architecture layer and unavoidable MCU startup.

## Workspace layout

```text
specs/          contracts — traits and minimal shared types
kernel/         hardware-blind orchestration: scheduling, task lifecycle
arch/           CPU/ISA mechanics: context switch, interrupt primitives
mcu/            per-MCU bring-up: reset, vector table, memory map
bsp/            board composition: binds arch + mcu + kernel, exposes the safe API
macros/         the entry-point attribute macro
phase_testing/  validation binaries, one per gatekeeper check
scripts/        build / flash / debug helpers
docs/           design records, runbooks, scope, and superseded planning
```

The authoritative boundary definitions are in [`docs/scope.md`](docs/scope.md).

## Building and testing

The default build target is embedded (set in `.cargo/config.toml`); install it with `rustup target add <triple>` first, or let the pinned `rust-toolchain.toml` do it.

- `cargo build` — the core crates.
- `cargo build --workspace` — everything, including the validation binaries.
- `cargo test -p kernel --target <host-triple>` — kernel host tests. A host target is required because the default target has no test harness.

## Using ThetOS from an external project

A firmware project outside this workspace depends on the entry-macro crate and a board-support crate by git, and builds a binary for its board. The recipe — manifest, `.cargo/config.toml`, and entry point — is in [`docs/external-usage.md`](docs/external-usage.md).

## Flashing and debugging

`scripts/` automate build, flash (via OpenOCD + a probe), and GDB attach for in-tree binaries. Configure the `[scripting]` section of `.cargo/config.toml` for your probe and OpenOCD install. The reproducible hardware-validation procedures are in [`docs/runbooks/`](docs/runbooks/).

## Documentation

- [`docs/scope.md`](docs/scope.md) — what v1.0.0 is, what it isn't, the architecture boundaries.
- [`docs/decisions/`](docs/decisions/) — architecture decision records: each choice, its alternatives, its consequences.
- [`docs/runbooks/`](docs/runbooks/) — reproducible gatekeeper validation procedures.
- [`docs/external-usage.md`](docs/external-usage.md) — depending on ThetOS from another project.
- [`docs/history/`](docs/history/) — the original six-phase plan and per-phase guides. **Superseded; kept for the thesis narrative, not current.**

## Design philosophy

**Static dispatch.** No `dyn Trait`. The kernel is generic over its hardware dependencies; monomorphisation inlines the concrete implementations, so the generated code matches a hand-written target-specific kernel.

**Compile-time invariant enforcement.** Hardware states are encoded in the type system, so a class of runtime logic errors — using a peripheral in the wrong state — becomes a build failure instead.
