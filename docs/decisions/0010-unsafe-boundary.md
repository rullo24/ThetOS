# ADR-0010: unsafe confined to arch/ and mcu startup

Status: Accepted

## Context
The thesis claims user-facing APIs are safe Rust. That is only credible if `unsafe` has a defined, small footprint.

## Decision
`unsafe` is allowed in `arch/` (context save/restore, PendSV/IRQ, stack guard) and in unavoidable `mcu/` startup (reset, RAM init, vector table, VTOR). Driver register access in `bsp/` is `unsafe` internally but wrapped in a safe API. Application code and examples are safe, with one sanctioned exception: the `static mut STACK_POOL` reference in an example's `app_main` (`&mut *addr_of_mut!(STACK_POOL)`), which cannot be abstracted away without a heap (explictly).

## Alternatives
- `#![forbid(unsafe_code)]` in `kernel/` and `bsp/`: desirable; blocked today by the BSP register pokes and the example stack pool. Revisit with a `StaticCell`-style wrapper.

## Consequences
- `unsafe` outside `arch/` and `mcu/` is a review signal.
- The stack-pool line is the known, documented exception; new app-side `unsafe` is not.
