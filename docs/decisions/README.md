# Architecture Decision Records

One file per decision. Body format: **Status / Context / Decision / Alternatives / Consequences**, kept short.

Supersede, do not edit: a reversed decision gets a new ADR; the old one's `Status` becomes `Superseded by ADR-NNNN` and its body stays as history.

| ADR | Decision | Status |
|-----|----------|--------|
| [0001](0001-static-dispatch.md) | Static dispatch, generic kernel | Accepted |
| [0002](0002-crate-boundaries.md) | Crate boundaries and dependency direction | Accepted |
| [0003](0003-manual-drivers.md) | Manual peripheral drivers, no HAL | Accepted |
| [0004](0004-fpp-scheduler.md) | Fixed-priority preemptive scheduler | Accepted (supersedes round-robin plan) |
| [0005](0005-blocking-and-idle-task.md) | Blocked state + idle task, not busy-wait | Accepted |
| [0006](0006-debug-tick-rate.md) | 100 Hz SysTick on debug builds | Accepted |
| [0007](0007-typestate-driver-api.md) | Typestate drivers, infallible GPIO API | Accepted |
| [0008](0008-gpio-structure.md) | GPIO: BSP registers, const-generic pins, per-port modules | Accepted |
| [0009](0009-system-singleton.md) | System as a global singleton via free functions | Accepted |
| [0010](0010-unsafe-boundary.md) | unsafe confined to arch/ and mcu startup | Accepted |
| [0011](0011-external-import.md) | External projects import ThetOS by git; linker scripts self-locate | Accepted |
| [0012](0012-flat-consumer-api.md) | Flat consumer API — `thetos_entry` + one board crate | Accepted |
| [0013](0013-v1-scope.md) | v1.0.0 scope — kernel proven, GPIO only | Accepted |
