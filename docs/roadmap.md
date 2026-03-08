# Thesis Roadmap: Modular Rust RTOS

## Overview
This roadmap outlines the development cycle for a modular Real-Time Operating System (RTOS) implemented in Rust. The primary objective is to prove that hardware-state invariants and memory safety can be enforced at compile-time via **Static Dispatch** and **Typestate Patterns** without sacrificing performance.

---

## Phase 1: The Formal Contract & Workspace (Expected: 4 weeks)
**Objective:** Establish the "Laws" of the system and the zero-touch build pipeline.

* **Tasks:**
    * Initialise the Cargo Workspace and `.cargo/config.toml` for `no_std` cross-compilation.
    * Define core Trait signatures in `specs/` for `ContextSwitch`, `SystemTimer`, `Uart`, and `Gpio`.
    * Implement the "Zero-Touch" dispatcher in `kernel/src/lib.rs` using `cfg` flags to alias the architecture crate.
* **Deliverables:**
    * **Trait `ContextSwitch`**: Signatures for stack frame initialisation and manual trigger of the context switch interrupt.
    * **Trait `SystemTimer`**: Signatures for frequency configuration, interrupt clearing, and tick-waiting.
    * **Trait `Uart`**: Signatures for baud-rate setup, atomic byte transmission, and byte reception.
    * **Trait `Gpio`**: Signatures for pin state manipulation and mode switching (Input/Output).
* **Documentation:** Draft the "Architectural Design" chapter justifying the trait-based approach over C-style header definitions.

* **Gatekeeper 1:** A workspace that successfully runs `cargo build --target thumbv7em-none-eabihf` where the `kernel` can call these API signatures on "stubbed" implementations.

---

## Phase 2: The Hardware "Port" (Expected: 5 weeks)
**Objective:** Implement the "Unsafe" core and register-level logic.

* **Tasks:**
    * Write the Assembly `PendSV` or `SysTick` handlers in `arch/arm-cortex-m`.
    * Implement manual stack frame allocation (creating the "Initial State" for a task).
    * Implement the `ContextSwitch` trait for the ARM architecture.
* **Deliverables:**
    * **Assembly Source**: Hard-coded register save/restore logic (`R0-R12`, `LR`, `PC`).
    * **Stack Initialiser**: A function that accepts a function pointer and returns a formatted stack pointer (`sp`).
* **Documentation:** Detailed diagramming of the Stack Anatomy and register save/restore sequences.

* **Gatekeeper 2:** A "Primitive Switch" demo where the CPU successfully jumps from `main()` to a single hard-coded function and back to a known point, verified via GDB register inspection.

---

## Phase 3: The Minimal Kernel & Scheduler (Expected: 5 weeks)
**Objective:** Orchestrate task execution through logic rather than manual jumps.

* **Tasks:**
    * Develop the `TaskControlBlock` (TCB) within the `kernel` crate.
    * Implement a **Round-Robin Scheduler** logic.
    * Integrate the `arch` context-switch logic with the `kernel` scheduler.
    * Implement a global "Critical Section" or Mutex to protect the Ready List.
* **Deliverables:**
    * **The `Scheduler` Object**: A structure managing a list of `TCB`s and the "Current Task" pointer.
    * **System Heartbeat**: Integration of `SystemTimer` to trigger the scheduler at a fixed interval (e.g., 1ms).
* **Documentation:** Flowcharts of the scheduling algorithm and state-transition diagrams for tasks (Ready, Running, Blocked).
* **Gatekeeper 3:** "Multi-Blinky" validation. Two independent tasks toggling distinct GPIO pins, managed by the scheduler, running on physical hardware.

---

## Phase 4: Safety-Critical Drivers (Expected: 5 weeks)
**Objective:** Prove the "Typestate" safety claims in the problem statement.

* **Tasks:**
    * Implement the STM32 UART driver in `drivers/uart` using the `Uart` trait.
    * Apply the **Typestate Pattern**: Ensure `write_byte()` is inaccessible until the `init()` method has returned a valid state.
    * Implement the GPIO driver with compile-time mode checking (preventing an Output-only method from being called on an Input-configured pin).
* **Deliverables:**
    * **Typestate Driver API**: Drivers that use generic states (e.g., `Pin<Input>` vs `Pin<Output>`) to restrict method availability.
    * **Error Handling**: Implementation of hardware-specific `Error` types associated with the `Uart` and `Gpio` traits.
* **Documentation:** Comparison of the Rust Driver code vs. a standard C HAL, highlighting where the compiler catches illegal state transitions.

* **Gatekeeper 4:** A "Safe Console" where the kernel logs its status over UART, and a test-case script proving the code fails to compile if the UART setup sequence is violated.

---

## Phase 5: Verification & Comparative Analysis (Expected: 5 weeks)
**Goal:** Collect the empirical data required for the 60-page thesis report.

* **Tasks:**
    * **The "Failure Suite":** Develop 5 intentional "Illegal" C programs (FreeRTOS) and 5 equivalent Rust programs.
    * **Benchmarking:** Measure Context-Switch latency in CPU cycles using a Logic Analyser or the DWT Cycle Counter.
    * **Size Analysis:** Compare the final binary footprint (.text, .data, .bss) against a minimal FreeRTOS build.
* **Deliverables:**
    * **Benchmark Suite**: A set of automated tests measuring cycle counts for the scheduler.
    * **Comparative Matrix**: A spreadsheet containing code-size and performance metrics.
* **Documentation:** Generate high-resolution graphs and tables for the "Evaluation" chapter.

* **Gatekeeper 5:** A completed data set comparing Rust RTOS vs. FreeRTOS, ready for academic review.

---

## Phase 6: Final Polish & Submission (Expected: Due date submission)
**Objective:** Finalise the "Plug and Play" evidence and complete the report.

* **Tasks:**
    * Finalise the `examples/` folder to demonstrate how a new board can be plugged in without changing kernel logic.
    * Complete the "Future Work" section regarding RISC-V or I2C/SPI support.
    * Final proofread and submission of the thesis document.
* **Final Deliverable:** * **Production Repository**: Fully commented, documented, and linted source code.
    * **Thesis PDF**: The submitted 60-page technical document.

---

## Summary of Milestones

| Milestone | Deliverable | Validation Method |
| :--- | :--- | :--- |
| **1** | Build Pipeline & API Specs | `cargo build` success for ARM target with stubbed traits. |
| **2** | Primitive Switch Handler | Validated CPU register state and PC jump in GDB. |
| **3** | Round-Robin Scheduler | Simultaneous task execution (Blinky) on hardware. |
| **4** | Typestate Drivers | Compilation failure on attempted illegal hardware state access. |
| **5** | Empirical Benchmark Report | Comparative cycle-count and binary-size analysis vs FreeRTOS (C). |