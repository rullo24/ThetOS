# ADR-0013: v1.0.0 scope — kernel proven, GPIO only

Status: Accepted

## Context
The thesis question is whether compile-time enforcement of hardware-state and memory-safety invariants is achievable in a modular Rust RTOS without a performance penalty. That question is answered by the kernel plus one typestate driver on real hardware. Building every peripheral driver is engineering breadth, not additional evidence.

## Decision
v1.0.0 delivers: the hardware-blind kernel, FPP scheduler, critical sections, blocking + idle task, `SystemTimer` heartbeat, the typestate GPIO driver, one board (Nucleo-L152RE), external consumption, and Gatekeeper 0–3.

Deferred to future versions, outside thesis scope: UART / PWM / I²C / SPI drivers, analog / ADC, GPIO alternate-function muxing, EXTI, more boards, a `thetos` facade crate, MPU-backed stack protection.

The RC-car demonstration (thesis Phase 5) is built as an **external consumer** of v1.0.0, not as more RTOS work.

## Alternatives
- Implement UART/PWM before tagging: needed for a richer RC-car demo, but adds no thesis evidence and delays the writeup. The car can run on GPIO-driven control.
- Never tag; keep everything on one branch: loses a reproducible reference for the external demo to pin.

## Consequences
- `v1.0.0` is a stable ref the RC-car repo pins.
- The gap between "thesis is about typestate" and "one typestate driver shipped" is owned here: the pattern is proven ([ADR-0007](0007-typestate-driver-api.md)); breadth is future work.
- Post-thesis work resumes by incrementing the version as peripherals land.
