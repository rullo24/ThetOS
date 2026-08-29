# ADR-0002: Crate boundaries and dependency direction

Status: Accepted

## Context
Mixing scheduling logic with register code is the usual source of untestable, unportable RTOS code. The kernel must compile for host and target unchanged.

## Decision
Five crates, fixed roles, one-way dependencies:
- `specs/` contracts only (traits + minimal shared types)
- `kernel/` hardware-blind orchestration; depends on `specs` only
- `arch/` CPU/ISA mechanics; depends on `specs`
- `mcu/` silicon bring-up (reset, memory map, vector table)
- `bsp/` board composition; depends on `kernel + arch + mcu + specs`

No reverse dependencies. Concrete structs never live in `specs/`. Full hard-defines and review checklist: [`../scope.md`](../scope.md).

## Alternatives
- Single crate with modules: no enforced boundary, register code leaks into the kernel.
- Per-peripheral HAL crates: more crates, still needs this core split.

## Consequences
- `cargo build -p kernel --target <host>` proves hardware-blindness mechanically.
- A new board is a new `bsp/` crate; kernel and specs untouched.
- A new capability often touches a `specs/` trait plus an impl crate.
