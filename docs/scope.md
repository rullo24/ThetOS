# ThetOS v1.0.0 — scope and status

The thesis question: can a modular RTOS in Rust enforce memory-safety and hardware-state invariants at **compile time** (static dispatch + typestate) without a performance penalty versus a C RTOS. v1.0.0 is the point where that is proven on hardware. Further peripherals are future work, outside the thesis scope.

## Delivered

- A hardware-blind kernel, generic over its hardware dependencies, with no dynamic dispatch. The same kernel compiles for a host target (for tests) and the embedded target.
- The crate architecture: contracts, kernel, architecture port, MCU bring-up, board support — with one-way dependencies and `unsafe` confined to the architecture and MCU-startup layers.
- A fixed-priority preemptive scheduler behind a policy trait, with FIFO ordering within a priority level.
- Closure-based critical sections, a PendSV-driven context switch, per-task stack-guard canaries.
- Real task blocking with wake-on-tick and an always-present idle task.
- An interrupt-driven system tick behind a trait.
- A typestate GPIO driver: a pin is unconfigured until given a direction, and direction-wrong operations do not compile.
- One board: the Nucleo-L152RE.
- A documented path for an external project to depend on ThetOS and build firmware.
- Host unit tests for the kernel, and reproducible hardware-validation runbooks.

Rationale and rejected alternatives for each of these are in [`decisions/`](decisions/).

## Architecture boundaries (authoritative)

- **`kernel/`** owns hardware-blind orchestration only: task lifecycle, scheduler-policy invocation, kernel state transitions, the safe core API. Never register-level or board-specific code.
- **`arch/`** owns CPU/ISA mechanics only: context-frame ABI, save/restore, PendSV/IRQ primitives, stack-guard internals. Never board mapping, startup policy, or user-facing ergonomics.
- **`mcu/`** owns device-family bring-up only: reset/startup, vector-table ownership, memory and link surfaces. Never scheduling policy or kernel orchestration.
- **`bsp/`** owns board composition only: bind a concrete architecture, MCU, and kernel; expose the safe board-facing API. Never generic kernel policy or low-level CPU internals.
- **`specs/`** holds contracts and minimal shared model types only; concrete implementation structs live in their owning crates.
- Dependencies flow one way toward `specs/`; there are no reverse dependencies.
- User-facing APIs are safe Rust.

## Methodology

Development was structured as phases, each closed by a **gatekeeper**: a concrete, reproducible pass/fail check on physical hardware. The gatekeepers live on as [`runbooks/`](runbooks/). The original phased plan is preserved, superseded, in [`history/`](history/).

## Evaluation criteria

1. Safety robustness versus a C equivalent.
2. Context-switch latency.
3. Binary footprint in a constrained target.
