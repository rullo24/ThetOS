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

**Rust Tooling:**
```bash
rustup component add llvm-tools-preview
cargo install cargo-binutils
cargo install cargo-generate
```

---

## 🛠 Technical Stack & Implementation Constraints
To ensure the academic validity of the thesis, the following boundaries are established:
* **Core Infrastructure:** Use the appropriate architecture runtime crates for the target (e.g. `cortex-m-rt` for ARM) for vector table and startup logic.
* **Hardware Access:** Use Peripheral Access Crates (PAC) for raw register definitions only.
* **Driver Layer:** **Manual Implementation.** High-level Hardware Abstraction Layers (HALs) are forbidden. All Typestate logic and Trait implementations must be original work to validate the safety claims of the thesis.
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
├── arch/                     # THE PORTS: CPU-specific assembly & context logic
├── drivers/                  # MODULES: System-level peripheral implementations
├── boards/                   # THE GLUE: Mapping kernel to physical hardware
├── scripts/                  # TOOLING: Linker scripts & debug configurations
└── examples/                 # PROOF: Sample applications for validation
```

The phased development plan, gatekeepers, and milestones are defined in [docs/roadmap.md](docs/roadmap.md).

## Building

**Prerequisites:** Install the Rust toolchain (e.g. via [rustup](https://rustup.rs)). For embedded targets, install the required target before building (the target is set in `.cargo/config.toml`). For example, for ARM Cortex-M3:

```bash
rustup target add thumbv7m-none-eabi
```

**Build commands:**

* **`cargo build`** — Builds only the RTOS library crates (kernel, specs, arch, drivers, boards). These are the crates listed in **`default-members`** in the root `Cargo.toml`. Add each new lib crate to both `members` and `default-members` as the project grows.
* **`cargo build --workspace`** — Builds every crate in the workspace, including all examples. This uses the full **`members`** list in the root `Cargo.toml`. Ensure `members` lists every crate (libs and examples); add each new example or lib there.

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

### 4. `drivers/` (Modular System Services)
Standardised, swappable peripheral modules.
* **Purpose:** Provides implementations for serial consoles and system timers that adhere to `specs/`.
* **Thesis Relevance:** Validates the **Typestate Pattern** by enforcing peripheral state machine safety (e.g., preventing a write to an uninitialised UART).

### 5. `boards/` (System Integration & BSP)
The "Matchmaker" layer defining the physical system configuration.
* **Purpose:** Maps specific pins to kernel functions and defines memory boundaries (SRAM/Flash).
* **Thesis Relevance:** Proves **System Composition** by allowing the same kernel to be deployed across diverse hardware targets.

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