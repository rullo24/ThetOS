# ThetOS - A Modular Real-Time Operating System (RTOS) in Rust

An investigation into compile-time safety invariants and zero-cost modularity for mechatronic systems.

---

## 📑 Thesis Problem Statement

> Current industry-standard RTOS architectures (typically C-based) lack the ability to enforce memory safety and hardware-state invariants at compile-time, resulting in runtime failures that are difficult to detect and debug. This thesis seeks to prove that a modular RTOS developed in Rust, utilising static dispatch and the typestate pattern, can eliminate these failure modes at the compilation stage without incurring a performance penalty compared to a C-based equivalent.

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

### 1. `specs/` (The Formal Contract)
Defines the **Traits** (interfaces) that serve as the "law" for the system.
* **Purpose:** Formalises behaviours for `ContextSwitch`, `InterruptController`, and `SystemTimer`.
* **Thesis Relevance:** Demonstrates **Interface Segregation**, ensuring the kernel remains hardware-agnostic.

### 2. `kernel/` (The Core Orchestrator)
The hardware-independent kernel logic.
* **Purpose:** Manages task scheduling (Ready-lists), synchronisation primitives, and lifecycle management.
* **Thesis Relevance:** Central site for proving that scheduling logic can be safely decoupled from register-level manipulation.

### 3. `arch/` (Architecture-Specific Ports)
Low-level implementations for specific CPU instruction sets (e.g., ARM Cortex-M, RISC-V).
* **Purpose:** Manages stack frame initialisation, register saving/restoring, and atomic operations.
* **Thesis Relevance:** Isolates the "unsafe" code required for context switching from the safe modular kernel.

### 4. `drivers/` (Modular System Services)
Standardised, swappable peripheral modules.
* **Purpose:** Provides implementations for serial consoles and system timers that adhere to `specs/`.
* **Thesis Relevance:** Validates the **Typestate Pattern** by enforcing peripheral state machine safety (e.g., preventing a write to an uninitialised UART).

### 5. `boards/` (System Integration & BSP)
The "Glue" layer defining the physical system configuration.
* **Purpose:** Maps specific pins to kernel functions and defines memory boundaries (SRAM/Flash).
* **Thesis Relevance:** Proves **System Composition** by allowing the same kernel to be deployed across diverse hardware targets.

### 6. `docs/` (Planning & Technical Specifications)
Centralised repository for design documents used to support the final thesis report.
* **`architecture.md`:** Definition of the modular layers and data flow.
* **`safety_invariants.md`:** Mapping of specific failure modes to Rust's compile-time checks.
* **`performance_metrics.md`:** Benchmarking methodology for context-switch latency.

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