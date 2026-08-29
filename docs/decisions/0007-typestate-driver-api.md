# ADR-0007: Typestate drivers with an infallible GPIO API

Status: Accepted

## Context
The thesis proves that illegal hardware-state access can be a compile error. A GPIO line has a configuration state (unset, input, output); operations are only valid in one of them.

## Decision
- Typestate: `Pin<.., Uninit>` then `into_input(pull)` / `into_output(style)` yields `Pin<.., Input>` / `Pin<.., Output>`. `read()` exists only on `Input`, `set()` only on `Output`. Markers are sealed. The wrong call does not compile.
- GPIO methods are infallible: `set(GpioLevel)` returns `()`, `read()` returns `GpioLevel`. No `Result`, no associated `Error` type.

## Alternatives
- Runtime mode check returning `Err(WrongMode)`: moves a compile error to runtime.
- Keep `Result` + `type Error` for future fallible backends (I2C expanders): speculative; every caller pays for error handling that cannot fire on a memory-mapped pin.

## Consequences
- Wrong-state access is a `trybuild` compile-fail case (Phase 4 Gatekeeper).
- A fallible GPIO backend would need its own trait or a breaking change here.
- No runtime direction switching: `into_*` consume `Uninit` only.
