# Phase 2 — Gatekeeper 2 runbook

This document is the **repeatable** procedure to validate **Gatekeeper 2** on physical hardware: build, flash, attach GDB, and interpret pass/fail. Definitions and scope: [docs/history/roadmap.md](../history/roadmap.md), [docs/history/phase2_hardware_port.md](../history/phase2_hardware_port.md).

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

  Example: `target/thumbv7m-none-eabi/debug/primitive_switch` (packages live under `phase_testing/` in-tree).

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

Phase 2 demos live under `phase_testing/` (workspace package name is still the Cargo crate name).

| Package | Role |
|--------|------|
| `primitive_switch` | **Roadmap Gatekeeper 2**: first switch from `main` into a single prepared task (`task_entry`). |
| `psp_to_psp` | **Extended check**: cooperative PSP↔PSP ping-pong between two tasks. |
| `pendsv_pending_probe` | PendSV pending path probe (optional; not the core Gatekeeper 2 definition). |
| `p2_stack_guard` | **Stack guard**: `CortexMStackGuard` canary initialisation, intact `check`, deliberate overwrite, `GuardCorrupted` on second `check`. |

## Pass / fail — `primitive_switch`

1. Flash and run, or use `debug.py` and breakpoints.
2. **Pass** if execution reaches **`task_entry`** (e.g. `break task_entry`, `continue`, or `where` shows PC in `task_entry`), and **`TASK_HEARTBEAT`** advances if you halt again later.
3. **Fail** if the CPU never leaves `main` in the intended way, hits **HardFault**, or never reaches `task_entry`.

Optional: in `PendSV_Handler`, **`$lr`** is typically **`0xfffffffd`** when the preempted thread used PSP (`EXC_RETURN`); **`x/8xw $psp`** shows the eight-word hardware frame on that stack.

## Pass / fail — `psp_to_psp`

1. Set breakpoints on lines that run **every** iteration (e.g. heartbeat updates), not only `loop {`.
2. **Pass** if **`HEARTBEAT_A`** and **`HEARTBEAT_B`** both increase over time and execution alternates between tasks; in `PendSV_Handler`, **`$lr == 0xfffffffd`** with **`x/8xw $psp`** consistent with the hardware frame on each task’s stack.
3. **Fail** if one heartbeat never changes, or you see repeated **HardFault** / hang.

## Stack guard evidence (`p2_stack_guard`)

This demo uses **`specs::arch::StackGuard`** with **`cortex_m::CortexMStackGuard`** in **canary** mode: it seeds the word at `stack_limit`, verifies `check`, deliberately overwrites that word, then expects **`StackGuardError::GuardCorrupted`**. It is **manual corruption** of the canary slot (not a natural stack overflow), but it proves the **detection path** on hardware.

1. **Build / flash / debug**

   ```bash
   python3 scripts/build.py -p p2_stack_guard
   python3 scripts/run.py -p p2_stack_guard
   python3 scripts/debug.py -p p2_stack_guard
   ```

2. **GDB (after `load`, or from `debug.py` prompt)**

   ```text
   break phase_testing/phase2/p2_stack_guard/src/main.rs:93
   continue
   print/x GUARD_DEMO_PHASE
   ```

   (Line **93** is the final `spin_loop`; if GDB maps lines differently after edits, break on `app_main` and run until that loop, or `break p2_stack_guard::app_main` and `continue` twice.)

3. **Pass / fail**

   - **Pass:** `GUARD_DEMO_PHASE == 0x2` — second `check` saw a corrupted canary.
   - **Init failure:** `0xff` — `initialise` returned an error (bad bounds).
   - **Unexpected:** `0xfe` — second `check` did not return `GuardCorrupted`.
   - **Too early:** `0x1` only — you stopped before the overwrite / second `check`.

4. **Optional:** `info address STACK` then `x/wx <addr>` — after the demo the first word at `stack_limit` is **intentionally** `0` (stomp before the failing `check`).

For the full Phase 2 evidence checklist, see [docs/history/phase2_hardware_port.md](../history/phase2_hardware_port.md).

## Related

- [docs/history/roadmap.md](../history/roadmap.md) — milestones and Gatekeeper wording.
- [docs/history/phase2_hardware_port.md](../history/phase2_hardware_port.md) — implementation detail and evidence checklist.
