# Phase 2 — Gatekeeper 2 runbook

This document is the **repeatable** procedure to validate **Gatekeeper 2** on physical hardware: build, flash, attach GDB, and interpret pass/fail. Definitions and scope: [docs/roadmap.md](../roadmap.md), [docs/phase2/phase2_hardware_port.md](../phase2/phase2_hardware_port.md).

## Prerequisites

- **Board** compatible with the OpenOCD target in `.cargo/config.toml` (default in-tree: STM32L1 via `target/stm32l1.cfg`).
- **Probe** (e.g. ST-Link) and USB cable.
- **Rust**: [rustup](https://rustup.rs); target from `.cargo/config.toml` (e.g. `thumbv7m-none-eabi`):

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

  Example: `target/thumbv7m-none-eabi/debug/primitive_switch`.

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

| Package | Role |
|--------|------|
| `primitive_switch` | **Roadmap Gatekeeper 2**: first switch from `main` into a single prepared task (`task_entry`). |
| `psp_to_psp` | **Extended check**: cooperative PSP↔PSP ping-pong between two tasks. |
| `pendsv_pending_probe` | PendSV pending path probe (optional; not the core Gatekeeper 2 definition). |

## Pass / fail — `primitive_switch`

1. Flash and run, or use `debug.py` and breakpoints.
2. **Pass** if execution reaches **`task_entry`** (e.g. `break task_entry`, `continue`, or `where` shows PC in `task_entry`), and **`TASK_HEARTBEAT`** advances if you halt again later.
3. **Fail** if the CPU never leaves `main` in the intended way, hits **HardFault**, or never reaches `task_entry`.

Optional: in `PendSV_Handler`, **`$lr`** is typically **`0xfffffffd`** when the preempted thread used PSP (`EXC_RETURN`); **`x/8xw $psp`** shows the eight-word hardware frame on that stack.

## Pass / fail — `psp_to_psp`

1. Set breakpoints on lines that run **every** iteration (e.g. heartbeat updates), not only `loop {`.
2. **Pass** if **`HEARTBEAT_A`** and **`HEARTBEAT_B`** both increase over time and execution alternates between tasks; in `PendSV_Handler`, **`$lr == 0xfffffffd`** with **`x/8xw $psp`** consistent with the hardware frame on each task’s stack.
3. **Fail** if one heartbeat never changes, or you see repeated **HardFault** / hang.

## Stack guard evidence

**Not covered here** until the stack-guard corruption scenario exists. Full Gatekeeper 2 evidence (per [docs/phase2/phase2_hardware_port.md](../phase2/phase2_hardware_port.md)) includes a demonstrable overflow/guard path; extend this runbook when that is implemented.

## Related

- [docs/roadmap.md](../roadmap.md) — milestones and Gatekeeper wording.
- [docs/phase2/phase2_hardware_port.md](../phase2/phase2_hardware_port.md) — implementation detail and evidence checklist.
