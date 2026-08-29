# Phase 3 — Gatekeeper 3 runbook

This document is the **repeatable** procedure to validate **Gatekeeper 3** on physical hardware: build, flash, attach GDB, and interpret pass/fail. Definitions and scope: [docs/history/roadmap.md](../history/roadmap.md), [docs/history/phase3_minimal_kernel.md](../history/phase3_minimal_kernel.md).

Roadmap Gatekeeper 3: *"Multi-Blinky" validation. Two independent tasks toggling distinct GPIO pins, managed by the scheduler, running on physical hardware with no race conditions.*

## Prerequisites

- **Board** compatible with the OpenOCD target in `.cargo/config.toml` (default in-tree: STM32L1 via `target/stm32l1.cfg`). Reference board: Nucleo-L152RE (STM32L152RET6).
- **Probe** (e.g. ST-Link) and USB cable.
- **External indicators**: an LED + series resistor on **PA6** and on **PA7**, or a two-channel scope / logic analyser. PA6/PA7 are on the CN10 morpho header (Arduino D12 / D11). Neither is an on-board LED.
- **Rust**: [rustup](https://rustup.rs); target from `.cargo/config.toml`:

  ```bash
  rustup target add thumbv7m-none-eabi
  ```

- **Python 3.11+** (for `scripts/*.py`).
- **OpenOCD** and **GDB** (`arm-none-eabi-gdb`, or the binary named in `.cargo/config.toml` under `[scripting] gdb_instance`).

Optional: set `openocd_interface`, `openocd_target`, and `openocd_scripts_dir` in `.cargo/config.toml` for your machine.

## Paths

- **Repository root**: all commands below assume `cwd` is the repo root.
- **ELF (debug)**:

  ```text
  target/thumbv7m-none-eabi/debug/<package_name>
  ```

  Example: `target/thumbv7m-none-eabi/debug/double_gpio_toggle` (packages live under `phase_testing/phase3/`).

## Build

```bash
python3 scripts/build.py -p <package>
```

Release (optional):

```bash
python3 scripts/build.py -p <package> --release
```

## Flash

```bash
python3 scripts/run.py -p <package>
```

Rebuilds then flashes unless `--no-build` is passed (see `scripts/run.py --help`).

## Debug (OpenOCD + GDB)

Starts OpenOCD in the background, launches GDB, loads the ELF, resets, breaks at `main`, and continues:

```bash
python3 scripts/debug.py -p <package>
```

Attach address is `extended-remote :3333`. For a manual session, run OpenOCD separately, then point GDB at the same ELF and use `target extended-remote :3333`, `monitor reset halt`, `load`.

## Which example for which check

Phase 3 demos live under `phase_testing/phase3/` (workspace package name is the Cargo crate name).

| Package | Role |
|--------|------|
| `double_gpio_toggle` | **Roadmap Gatekeeper 3**: two same-priority tasks toggling **PA6** and **PA7** independently under the scheduler. |
| `blinking_leds` | Single-task GPIO bring-up: one task toggling **PA5** (LD2) via `delay_ms`. |
| `multi_task_auto` | Three same-priority tasks, no delays — FIFO / round-robin within one priority level (`COUNTER_A/B/C` advance together). |
| `five_task_priority_demo` | Five tasks across priorities 0/7/15/23/31 — strict fixed-priority ordering, `delay_ms` opens windows for lower tiers. |
| `priority_delay_demo` | Three tasks (0/15/31) — `delay_ms` on the higher tiers lets the lowest, never-delaying task run. |
| `all_tasks_delay_demo` | Two same-priority tasks both sleeping — exercises the idle-task fallback (see note below). |
| `systick_tick_probe` | Single task — SysTick fires, `reschedule` reselects the same task, PendSV never triggers. |

> **Note — `all_tasks_delay_demo`:** its source comment says "there is no idle task", but `System::new_with_pool` now auto-spawns one (`TaskId(0)`). With the idle task present, `delay_ms` no longer returns `NoRunnableTask`, so `DELAY_REFUSED_COUNT` is expected to stay `0` and idle runs in the gaps. Treat a non-zero `DELAY_REFUSED_COUNT` as a regression, and update the comment when convenient.

## Pass / fail — `double_gpio_toggle` (Gatekeeper 3)

1. Build and flash:

   ```bash
   python3 scripts/run.py -p double_gpio_toggle
   ```

2. With the board **free-running** (no debugger), observe PA6 and PA7:
   - **Pass** if **both** pins toggle continuously and independently — roughly a 2 s period each (≈1 s high, ≈1 s low), driven by `system::delay_ms(1000)` at a 100 Hz debug-build tick.
   - **Fail** if only one pin moves, neither moves, the board **HardFaults**, or it hangs.

3. Under GDB (`scripts/debug.py -p double_gpio_toggle`):
   - `break double_gpio_toggle::toggle_task_a` and `break double_gpio_toggle::toggle_task_b`, then `continue` repeatedly — **both** breakpoints must be hit over time (proves two independent contexts run).
   - In `PendSV_Handler`, `$lr` is `0xfffffffd` when the preempted thread used PSP (`EXC_RETURN`); `x/8xw $psp` shows the eight-word hardware frame on that task's stack.
   - `print TICK_CYCLES_MAX` after some run time — see the tick-budget check below.

4. Optional register inspection (GPIOA base `0x4002_0000`):

   ```text
   x/1xw 0x40020000   # GPIOA_MODER: bits [13:12] and [15:14] == 0b01 (PA6, PA7 = output)
   x/1xw 0x40020014   # GPIOA_ODR:   bits 6 and 7 flip between successive halts
   ```

   `GPIOA_BSRR` (`0x4002_0018`) is write-only and reads as `0`.

## Why there are no race conditions

State this in the evidence pack ([gatekeeper3_evidence.md](../gatekeeper3_evidence.md)); no extra code proves it:

- Each task owns a **distinct** `Pin<PortA, N, Output>` — no shared pin object.
- Level changes go through `GPIOx_BSRR`, a write-only register with one bit per set/reset action — never a read-modify-write on `ODR` — so a context switch between the two tasks' writes cannot lose or corrupt either pin.
- Kernel ready-list and scheduler-state mutations run inside `CortexMCriticalSection` (PRIMASK mask/restore). SysTick and PendSV are configured at the same lowest exception priority, so the tick handler and the context switch do not nest destructively.
- Task stacks are disjoint regions of the pool; the idle task has its own stack; a stack-guard canary is seeded per task.

## Host tests (must be green)

Gatekeeper 3 requires the scheduler host tests to pass on a desktop target:

```bash
cargo test -p kernel --target aarch64-apple-darwin
```

Expected: `fpp_scheduler_host` 11/11, `kernel_core_host` 25/25, `task_priority` 5/5.

## Tick-budget check (Phase 3 regression guard)

The debug (unoptimised) build's `on_tick_interrupt()` is expensive. `bsp/.../system.rs` runs SysTick at **100 Hz** (`SYSTICK_PERIOD_MS = 10`) for exactly this reason; the reload value is `SYSCLK_HZ * 10 / 1000 - 1 = 41942` at the 4.194 MHz MSI range.

After a run, halt and:

```text
print/x TICK_CYCLES_MAX
```

- **Pass:** well under `41942` (comfortable headroom for thread-mode code).
- **Fail:** at or above the reload — the tick handler cannot finish before the next tick and starves the tasks (the Phase 3 `#41` failure mode). Do not drop the tick period below 10 ms on a debug build.

## Related

- [docs/history/roadmap.md](../history/roadmap.md) — milestones and Gatekeeper wording.
- [docs/history/phase3_minimal_kernel.md](../history/phase3_minimal_kernel.md) — implementation detail and the Gatekeeper 3 evidence package.
- [docs/runbooks/phase2_gatekeeper.md](phase2_gatekeeper.md) — Phase 2 procedure (context-switch and stack-guard prerequisites).
