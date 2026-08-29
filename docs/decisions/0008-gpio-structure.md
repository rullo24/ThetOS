# ADR-0008: GPIO layout: BSP-owned registers, const-generic pins, per-port modules

Status: Accepted

## Context
STM32 GPIO is regular: a pin is (port base, pin index); every other field is arithmetic. Register access is die-specific but not board-specific.

## Decision
- Raw register access lives in `bsp/.../gpio/`, not `mcu/`. `mcu/` owns startup and the memory map; the pin driver is a peripheral driver the board composes.
- `Pin<PORT: GpioPort, const PIN_INDEX: u8, MODE>` is a zero-sized handle. All logic is generic; addresses fold to constants.
- One module per port under `gpio/port/` (`port_a.rs` ...), each a one-line `define_port!` giving base + RCC enable bit. `pin.rs` holds no addresses.
- Board pin handles are `const` ZSTs in `pins.rs` (`pub const PA5: Pin<PortA, 5, Uninit>`).

## Alternatives
- Register code in `mcu/`: allowed by the roadmap, but the driver is composition, not bring-up; `clock.rs` sets the RCC-poke precedent for genuine bring-up only.
- Per-pin files (`PA5.rs`): ~176 near-empty files holding two numbers each.
- Runtime `Pin { port, pin }` struct: 2 bytes, no compile-time pin identity; weaker for AF-mux driver signatures later.

## Consequences
- Any `Pin<PortX, N, Uninit>` works with no new code once the port module exists.
- No ownership guard: two handles to one physical pin compile. Accepted for now.
- GPIOF/GPIOG omitted (not bonded on the LQFP64 package).
