# ThetOS - A Modular Real-Time Operating System (RTOS) in Rust

An investigation into compile-time safety invariants and zero-cost modularity for mechatronic systems.

---

## 📑 Thesis Problem Statement

> Current industry-standard RTOS architectures (typically C-based) lack the ability to enforce memory safety and hardware-state invariants at compile-time, resulting in runtime failures that are difficult to detect and debug. This thesis seeks to prove that a modular RTOS developed in Rust, utilising static dispatch and the typestate pattern, can eliminate these failure modes at the compilation stage without incurring a noticeable performance penalty compared to a C-based equivalent.

## Software Dependencies

**Build (required for compiling):**
* **Rust** — Install via [rustup](https://rustup.rs). The target triple (e.g. `thumbv7m-none-eabi`) is set in `.cargo/config.toml`; install it with `rustup target add <target>` before building (see [Building](#building)).
* **Python 3.11+**

**Flashing and debugging (optional; only if you flash or attach GDB to hardware):**
* **OpenOCD** — Talks to the board over a compatible probe (e.g. ST-Link). Configure `[scripting]` in `.cargo/config.toml` with `openocd_interface` and `openocd_target` for your board.
* **A GDB Install** — For `scripts/debug.py` (attach, load, breakpoints).

### GDB Install (Fedora Example)

```bash
sudo dnf install openocd stlink gdb
```

**Rust Tooling:**
```bash
rustup component add llvm-tools-preview
cargo install cargo-binutils
cargo install cargo-generate
rustup target add thumbv7m-none-eabi
```

**

# Prerequisites (Fedora)
- sudo dnf install openocd stlink gdb-multiarch
- rustup target add thumbv7m-none-eabi

Build
- cd /path/to/no_rtos_no_config
- cargo build

Flash
- openocd -f /usr/share/openocd/scripts/interface/stlink.cfg \
          -f /usr/share/openocd/scripts/target/stm32l1.cfg \
          -c "add_script_search_dir /usr/share/openocd/scripts/target" \
          -c "program /path/to/no_rtos_no_config/target/thumbv7m-none-eabi/debug/no_rtos_no_config verify reset exit"

Debug (GDB)
- Terminal 1: run openocd without `exit` (keep it alive)
- Terminal 2: gdb-multiarch /path/to/no_rtos_no_config/target/thumbv7m-none-eabi/debug/no_rtos_no_config
- In GDB:
  target extended-remote :3333
  monitor reset halt
  load
  break main
  continue

---

## 🛠 Technical Stack & Implementation Constraints
To ensure the academic validity of the thesis, the following boundaries are established:
* **Core Infrastructure:** Use the appropriate architecture runtime crates for the target (e.g. `cortex-m-rt` for ARM) for vector table and startup logic.
* **Hardware Access:** Use Peripheral Access Crates (PAC) for raw register definitions only.
* **Peripheral drivers:** **Manual implementation.** High-level Hardware Abstraction Layers (HALs) are forbidden. All typestate logic and trait implementations must be original work to validate the safety claims of the thesis.
* **Kernel Layer:** Hardware-blind and generic-first. No architecture-specific code allowed in the `kernel/` crate.

---

## Project Architecture

The system is organised as a **Cargo Workspace** to enforce strict compile-time boundaries between the hardware-independent kernel and the target-specific hardware ports.

```text
/
├── Cargo.toml                # Workspace manifest (links all crates)
├── docs/                     # PLANNING: Design specs & architectural diagrams
├── kernel/                   # THE CORE: Hardware-independent scheduling logic
├── specs/                    # THE CONTRACTS: Traits for modules to implement
├── arch/                     # THE PORTS: CPU-specific assembly, linker layout, context logic
├── mcu/                      # DEVICE: Per-MCU crates (startup, memory.x, device link surface)
├── bsp/                      # BOARD: BSP crates that wire an MCU crate to a physical board
├── macros/                   # PROC MACROS: Attributes such as `#[entry]`
├── scripts/                  # TOOLING: Build, flash, and debug helpers
└── examples/                 # PROOF: Sample applications for validation
```

The phased development plan, gatekeepers, and milestones are defined in [docs/roadmap.md](docs/roadmap.md).
Authoritative module boundaries are defined in `docs/roadmap.md` under `Architecture Boundaries`.

## Building

**Prerequisites:** Install the Rust toolchain (e.g. via [rustup](https://rustup.rs)). For embedded targets, install the required target before building (the target is set in `.cargo/config.toml`). For example, for ARM Cortex-M3:

```bash
rustup target add thumbv7m-none-eabi
```

**Build commands:**

* **`cargo build`** — Builds the crates listed in **`default-members`** in the root `Cargo.toml` (see that file for the current set; RTOS pieces such as `kernel/`, `specs/`, `mcu/`, and `bsp/` are added there as they come online). Add each new library crate to both `members` and `default-members` as the project grows.
* **`cargo build --workspace`** — Builds every crate in the workspace, including all examples. This uses the full **`members`** list in the root `Cargo.toml`. Ensure `members` lists every crate (libs and examples); add each new example or lib there.
* **`cargo test -p kernel --target aarch64-apple-darwin`** — Runs kernel host tests on a desktop target (required because the default project target is embedded and cannot run test harnesses).

## Developer Workflow & Configuration

ThetOS is designed for **Declarative Configuration**. To minimise "Silent Failures" the developer is shielded from the internal complexity of the `arch/` and `kernel/` crates. Configuration is centralised into two specific files:

### 1. The Build Target (`/.cargo/config.toml`)
**Role:** Defines **"Where"** the code is going.  
This file contains the hardware metadata: the target triple for the chosen MCU and the path to the linker script (memory map). Once set for a given board, this file remains **static**.

### 2. The Feature Manifest (`/Cargo.toml`)
**Role:** Defines **"What"** the hardware is capable of.  
The developer interacts exclusively with the root `Cargo.toml` to toggle system-wide capabilities. Using Rust's **Feature Bubbling**, selecting a feature at the root (e.g. a capability flag) automatically triggers the corresponding code paths in the `arch/` layer.

> **Key Advantage:** The developer never modifies the RTOS source code to suit their hardware. By declaring the hardware features in the manifest, the compiler automatically reconfigures the context-switching logic and peripheral drivers at build-time.

---

## Module Breakdown

### 1. `specs/` (The Formal Contract)
Defines the **Traits** (interfaces) that serve as the "law" for the system.
* **Purpose:** Formalises behaviours for `ContextSwitch`, `SystemTimer`, `Uart`, and `Gpio`; plus typestate machine traits (e.g., `State`, `Uninitialized`, `Enabled`).
* **Thesis Relevance:** Demonstrates **Interface Segregation**, ensuring the kernel remains hardware-agnostic.

### 2. `kernel/` (The Core Orchestrator)
The hardware-independent kernel logic.
* **Purpose:** Manages task scheduling (Ready-lists), synchronisation primitives, and lifecycle management.
* **Thesis Relevance:** Central site for proving that scheduling logic can be safely decoupled from register-level manipulation.

### 3. `arch/` (Architecture-Specific Ports)
Low-level implementations for supported CPU instruction sets.
* **Purpose:** Manages stack frame initialisation, register saving/restoring, and atomic operations.
* **Thesis Relevance:** Isolates the "unsafe" code required for context switching from the safe modular kernel.

### 4. `mcu/` (Device crates)
Per-microcontroller packages: reset vector, RAM/flash map (`memory.x`), and other bring-up that is specific to a die or family (not a full development board).
* **Purpose:** Owns the **device** side of linking and startup so examples and BSP crates can depend on one MCU crate rather than duplicating linker and reset logic.
* **Thesis Relevance:** Keeps **PAC-level** and **vendor-specific** details out of the generic kernel while remaining explicit about which silicon is targeted.

### 5. `bsp/` (Board Support Packages)
Board-level crates (e.g. a Nucleo board) that depend on an `mcu/` crate and expose the wiring and dependencies applications use.
* **Purpose:** Maps a concrete board to the MCU crate and pulls the right artefacts into the link (pins, optional probes, feature flags).
* **Thesis Relevance:** Proves **system composition** by letting the same kernel and `specs/` contracts target different boards via different BSP crates.

### 6. `docs/` (Planning & Technical Specifications)
Centralised repository for design documents used to support the final thesis report (e.g. roadmap, architecture, safety invariants, benchmarking methodology).

---

## Design Philosophy

### Static Dispatch for Real-Time Rigour
To ensure deterministic execution, the RTOS avoids dynamic dispatch (`dyn Trait`) and virtual tables. Utilising **Monomorphisation**, the compiler inlines implementations at the call site, matching the performance of hand-optimised C.

### Compile-Time Invariant Enforcement
By encoding hardware states into the type system (Typestates), the RTOS transforms common runtime logic errors into build-time failures. This ensures that the system is "Correct by Construction".

---

## Evaluation Metrics
The project will be empirically validated against the following criteria:
1. **Safety Robustness:** Comparison of error-catching capabilities against a C-based equivalent (FreeRTOS).
2. **Context-Switch Latency:** Cycle-accurate measurement of scheduling overhead.
3. **Binary Footprint:** Analysis of binary bloat vs performance gains in a resource-constrained environment.