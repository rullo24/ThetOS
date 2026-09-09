# ADR-0014: UART driver, polled and single-owner

Status: Accepted

## Context
The RC-car demonstration ([ADR-0013](0013-v1-scope.md)) needs serial comms. UART was deferred from the thesis tag; this is the first peripheral added afterward. The driver has to fit the existing typestate pattern ([ADR-0007](0007-typestate-driver-api.md)) and the manual, no-HAL rule ([ADR-0003](0003-manual-drivers.md)).

## Decision
A typestate `Serial` handle in the BSP, `Uninit` until `into_active(UartConfig)` programs the hardware, mirroring the GPIO driver. Activation consumes the two USART2 pins as unconfigured GPIO handles, so a pin cannot be both a UART line and a GPIO. The contract lives in `specs/`.

The driver is **polled**: blocking byte writes, and a non-blocking read that returns "no byte yet" as `Ok(None)`, distinct from a receive fault. No interrupts, no DMA, no ring buffers.

One owner: a single `Serial` handle, no split into independent transmit and receive halves. The board exposes only USART2, the instance wired to the on-board debug probe's virtual COM port.

## Alternatives
- Interrupt plus receive ring buffer: required to guarantee no dropped bytes under load. Deferred until a workload needs it; with polling, a slow reader surfaces as an observable overrun rather than silent loss.
- Split transmit/receive handles: doubles the API surface and invites two tasks polling the same receiver. Move semantics already give a single owner for free.
- Reuse `embedded-hal` traits: rejected for the same reason as [ADR-0003](0003-manual-drivers.md), the project defines its own contracts.

## Consequences
- A consumer needing loss-free receive must poll often enough; overrun is reported, not hidden.
- `specs/` gains a UART contract beside the GPIO one; the BSP gains a `uart` module.
- First use of the GPIO `Alternate` typestate marker.
- Hardware validated by an echo runbook.
