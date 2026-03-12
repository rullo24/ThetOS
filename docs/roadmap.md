# Thesis Roadmap: Modular Rust RTOS

## Overview
This roadmap outlines the development cycle for a modular Real-Time Operating System (RTOS) implemented in Rust. The primary objective is to prove that hardware-state invariants and memory safety can be enforced at compile-time via **Static Dispatch** and **Typestate Patterns** without sacrificing performance.

---

## 📑 Thesis Problem Statement

> Current industry-standard RTOS architectures (typically C-based) lack the ability to enforce memory safety and hardware-state invariants at compile-time, resulting in runtime failures that are difficult to detect and debug. This thesis seeks to prove that a modular RTOS developed in Rust, utilising static dispatch and the typestate pattern, can eliminate these failure modes at the compilation stage without incurring a noticable performance penalty compared to a C-based equivalent.

---

## 🛠 Technical Stack & Implementation Constraints
To ensure the academic validity of the thesis, the following boundaries are established:
* **Core Infrastructure:** Use `cortex-m` and `cortex-m-rt` for standard vector table and startup logic.
* **Hardware Access:** Use Peripheral Access Crates (PAC) for raw register definitions only.
* **Driver Layer:** **Manual Implementation.** High-level Hardware Abstraction Layers (HALs) are forbidden. All Typestate logic and Trait implementations must be original work to validate the safety claims of the thesis.
* **Kernel Layer:** Hardware-blind and generic-first. No architecture-specific code allowed in the `kernel/` crate.

---

## Phase 0: The Hardware Sanity Check (Expected: 1-2 weeks)
**Objective:** Establish a "Bare-Metal" baseline and verify the toolchain/debug pipeline.

* **Tasks:**
    * Configure the `memory.x` linker script for the target board's Flash and RAM boundaries.
    * Implement a minimal `no_std` entry point using the relevant architecture runtime (e.g., `cortex-m-rt`).
    * Implement a custom `#[panic_handler]` that triggers a hardware "Safe State" (e.g., LED SOS).
    * Set up the debug environment: `probe-rs`, `openocd`, or GDB integration for flashing and real-time inspection.
* **Deliverables:**
    * **Blinky/Panic Baseline**: A minimal program that toggles a GPIO or increments a volatile variable.
    * **Custom Panic Handler**: Verified "Safe-Halt" mechanism on code panic.
    * **The "Debug Stack"**: A working `.cargo/config.toml` that allows for `cargo run` execution directly to hardware.
* **Tests that must pass:** None (validation is manual; see Gatekeeper 0).
* **Gatekeeper 0:** Successful code execution on physical hardware verified via LED toggle or GDB register inspection.

---

## Phase 1: The Formal Contract & Workspace (Expected: 4 weeks)
**Objective:** Establish the "Laws" of the system and the generic build pipeline.

* **Tasks:**
    * Initialise the Cargo Workspace with a dedicated `boards/` crate to act as the system "Matchmaker."
    * Define core Trait signatures in `specs/` for `ContextSwitch`, `SystemTimer`, `Uart`, and `Gpio`.
    * Define the **Typestate Machine** traits in `specs/` (e.g., `trait State`, `struct Uninitialized`, `struct Enabled`).
    * Refactor Kernel Entry: Implement the kernel as a generic structure `Kernel<C: ContextSwitch>` that accepts hardware at instantiation.
* **Deliverables:**
    * **Trait ContextSwitch**: Signatures for stack frame initialisation and context switch triggers.
    * **Generic Kernel Skeleton**: A hardware-blind `lib.rs` that compiles for any target.
    * **Host Unit Tests**: A set of tests verifying scheduling logic on the host without hardware.
* **Tests that must pass:** `cargo test -p kernel` (or equivalent so that all kernel crate tests run) succeeds on a host target (e.g. `x86_64-unknown-linux-gnu` or `aarch64-apple-darwin`). At least: (1) the kernel builds and is instantiable with a mock implementation of `ContextSwitch`; (2) one or more `#[test]` functions exercise scheduling/ready-list logic using mock traits. No tests require hardware or an embedded target to run.
* **Gatekeeper 1:** Successful compilation of the same kernel logic for both a thumbv7 (ARM) target and an aarch64 (Host) target using mock traits.

---

## Phase 2: The Hardware "Port" (Expected: 5 weeks)
**Objective:** Implement the "Unsafe" core and register-level logic.

* **Tasks:**
    * Write the Assembly `PendSV` or `SysTick` handlers in `arch/arm-cortex-m`.
    * Implement manual stack frame allocation (creating the "Initial State" for a task).
    * Configure basic **Stack Overflow Detection** (utilising MPU or Watermarking).
    * Implement the `ContextSwitch` trait for the ARM architecture.
* **Deliverables:**
    * **Assembly Source**: Hard-coded register save/restore logic (`R0-R12`, `LR`, `PC`).
    * **Stack Guard Mechanism**: Verified detection of stack corruption via GDB.
* **Tests that must pass:** None (validation is via GDB and hardware; see Gatekeeper 2).
* **Gatekeeper 2:** A "Primitive Switch" demo where the CPU successfully jumps from `main()` to a single hard-coded function, verified via GDB register inspection.

---

## Phase 3: The Minimal Kernel & Scheduler (Expected: 5 weeks)
**Objective:** Orchestrate task execution through hardware-blind logic while ensuring kernel-level atomicity.

* **Tasks:**
    * **Develop the TaskControlBlock (TCB)**: Define a generic structure to store stack pointers and task states.
    * **Implement Critical Section Abstraction**: Define a `CriticalSection` trait in `specs/` using closure-based execution (`enter<F, R>(f: F)`) for atomic access.
    * **Implement Architecture-Specific Atomics**: Implement `CriticalSection` in `arch/` using global interrupt disabling (e.g., `cpsid i` / `cpsie i`).
    * **Develop Generic Round-Robin Scheduler**: Implement scheduling logic in `kernel/` utilising the `CriticalSection` trait to protect the Ready List.
    * **Integrate System Heartbeat**: Utilise the generic `SystemTimer` trait to trigger the scheduler via an interrupt-driven tick.
* **Deliverables:**
    * **The Scheduler Object**: A hardware-blind orchestration structure managing a collection of `TCBs`.
    * **Atomic Kernel Accessor**: A verified `CriticalSection` implementation that guarantees thread-safety for kernel structures.
    * **System Heartbeat**: Working implementation of `SystemTimer` triggering the scheduler's `yield` logic.
* **Tests that must pass:** `cargo test -p kernel` passes on a host target. At least: (1) unit tests for ready-list behaviour (add task, remove/select next, ordering) using mock `ContextSwitch` and mock `CriticalSection`; (2) unit tests that the scheduler invokes the context switch trait as expected. Multi-blinky on hardware remains manual (see Gatekeeper 3).
* **Gatekeeper 3:** "Multi-Blinky" validation. Two independent tasks toggling distinct GPIO pins, managed by the scheduler, running on physical hardware with no race conditions.

---

## Phase 4: Safety-Critical Drivers (Expected: 5 weeks)
**Objective:** Prove the "Typestate" safety claims in the problem statement.

* **Tasks:**
    * Implement the STM32 UART driver in `drivers/uart` using the `Uart` trait.
    * **Apply the Typestate Pattern**: Ensure `write_byte()` is inaccessible until the `init()` method has returned a valid state.
    * Implement the GPIO driver with compile-time mode checking.
* **Deliverables:**
    * **Typestate Driver API**: Drivers that use generic states (e.g., `Pin<Input>` vs `Pin<Output>`) to restrict method availability.
    * **Error Handling**: Implementation of hardware-specific `Error` types associated with the `Uart` and `Gpio` traits.
* **Tests that must pass:** A compile-fail test (e.g. using `trybuild` or a `tests/` crate that expects compilation failure) that verifies: code that calls `write_byte()` (or equivalent) before completing the UART `init()` sequence does not compile. Running `cargo test` (or the script that runs the compile-fail check) passes when this illegal usage is rejected by the compiler.
* **Gatekeeper 4:** Test-case script proving the code fails to compile if the UART setup sequence is violated.

---

## Phase 5: Verification & Comparative Analysis (Expected: 5 weeks)
**Goal:** Collect the empirical data required for the 60-page thesis report.

* **Tasks:**
    * **The "Failure Suite"**: Develop 5 intentional "Illegal" C programs (FreeRTOS) and 5 equivalent Rust programs.
    * **Benchmarking**: Measure Context-Switch latency in CPU cycles using a Logic Analyser or the DWT Cycle Counter.
    * **Static Dispatch Proof**: Use `cargo-bloat` or `objdump` to prove that generic traits were inlined with zero runtime overhead.
* **Deliverables:**
    * **Comparative Matrix**: A spreadsheet containing code-size and performance metrics vs. FreeRTOS.
* **Tests that must pass:** No new automated test suite; deliverables are data and methodology. The failure suite and benchmark procedure must be documented and reproducible so results can be re-run for review.
* **Gatekeeper 5:** A completed data set comparing Rust RTOS vs. FreeRTOS, ready for academic review.

---

## Phase 6: Final Polish & Submission (Expected: Due date submission)
**Objective:** Finalise the "Plug and Play" evidence and complete the report.

* **Tasks:**
    * Finalise the `examples/` folder to demonstrate how a new board can be plugged in without changing kernel logic.
    * Final proofread and submission of the thesis document.
* **Final Deliverable:**
    * **Production Repository**: Fully commented, documented, and linted source code.
    * **Thesis PDF**: The submitted 60-page technical document.
* **Tests that must pass:** All tests from Phases 1, 3, and 4 continue to pass; no regressions.

---

## Summary of Milestones

| Milestone | Deliverable | Validation Method |
| :--- | :--- | :--- |
| **0** | Hardware Sanity & Panic Handler | LED Blink + Verified "Safe Halt" on code panic. |
| **1** | Build Pipeline & Host Unit Tests | `cargo test` success on x86 for kernel logic. |
| **2** | Primitive Switch & Stack Guard | Validated CPU register state + Verified Fault on Stack Overflow. |
| **3** | Round-Robin Scheduler & Atomics | Simultaneous task execution + Race-condition verification. |
| **4** | Typestate Drivers | Compilation failure on attempted illegal hardware state access. |
| **5** | Empirical Benchmark Report | Comparative cycle-count and binary-size analysis vs FreeRTOS (C). |