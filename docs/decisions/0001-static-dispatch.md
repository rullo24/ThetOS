# ADR-0001: Static dispatch, generic kernel

Status: Accepted

## Context
The thesis claims determinism and C-equal performance. Dynamic dispatch (`dyn Trait`, vtables) adds an indirect call, blocks inlining, and is hard to bound.

## Decision
No `dyn Trait` on the hot path. The kernel is generic over its hardware dependencies: `Kernel<CtxSwitch, Crit, Scheduler, StackGuard, SystemTimer>`. The BSP picks concrete types at instantiation; the compiler monomorphises and inlines.

## Alternatives
- `dyn` trait objects: simpler signatures, one unbounded indirect call per switch.
- Feature-gated concrete types: no generics, but couples the kernel to each target.

## Consequences
- Generated code matches a hand-written target-specific kernel.
- Longer type signatures; one monomorphised kernel per target (acceptable).
- Mock trait impls give full host unit-test coverage with no hardware.
