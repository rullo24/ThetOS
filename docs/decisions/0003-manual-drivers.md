# ADR-0003: Manual peripheral drivers, no HAL

Status: Accepted

## Context
The thesis validates original typestate and trait design. Reusing `stm32l1xx-hal` or `embedded-hal` impls would mean the safety properties under test are someone else's work.

## Decision
Peripheral drivers are written from raw registers. High-level HALs are forbidden. Architecture runtime crates (vector table, startup) are allowed. A PAC is permitted for register definitions but is not currently used: addresses are hand-written from RM0038 with section citations.

## Alternatives
- Build on `embedded-hal` traits: faster, but the contribution becomes integration, not design.
- Vendor HAL (Cube): not idiomatic Rust, defeats the premise.

## Consequences
- Every driver (GPIO now; UART/PWM later) is written and owned here.
- Slower to reach breadth; each peripheral is a deliberate exercise.
- The safety claims cover code in this repo end to end.
