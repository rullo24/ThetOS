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

## Architecture Boundaries
* **`kernel/` owns orchestration only:** task lifecycle, scheduler invocation points, and kernel state transitions.
* **`arch/` owns CPU primitives only:** context frame setup, PendSV/interrupt mechanics, and stack guard internals.
* **`mcu/` owns silicon bring-up only:** startup/reset/vector table, memory map, and device-specific link surfaces.
* **`bsp/` owns board composition only:** wires `kernel + arch + mcu` and exposes the safe board-facing system API.
* **Dependency direction is strict:** `kernel -> specs`, `arch -> specs`, `bsp -> kernel + arch + mcu + specs`; no reverse dependencies.
* **Safety boundary is strict:** user-facing APIs must be safe Rust; `unsafe` is confined to `arch/` and unavoidable `mcu/` startup code.

### Hard Defines (Authoritative)
* **`arch/` hard define:** Owns CPU/ISA mechanics only (context frame ABI, context save/restore, PendSV/IRQ primitives, stack guard internals). Must never own board mapping, startup policy, or user-facing ergonomics.
* **`mcu/` hard define:** Owns device-family bring-up only (reset/startup, vector table ownership, memory/link surfaces, die-specific hooks). Must never own scheduling policy or kernel orchestration logic.
* **`kernel/` hard define:** Owns hardware-blind orchestration only (task lifecycle, scheduler policy invocation, kernel state transitions, safe core API). Must never own concrete register-level or board-specific implementation.
* **`bsp/` hard define:** Owns board composition only (bind concrete `arch + mcu + kernel`, select defaults, expose safe board-facing system entrypoints). Must never own generic kernel policy logic or low-level CPU save/restore internals.
* **`specs/` contract rule:** `specs/` contains contracts and minimal shared model types only; concrete implementation structs belong in their owning implementation crates (for example, concrete kernel models in `kernel/`).

### Boundary Review Checklist
* **Kernel purity check:** no board/device register logic or board resource mapping appears in `kernel/`.
* **MCU contract check:** `mcu` contracts describe silicon capabilities only (no app-facing ergonomics).
* **BSP composition check:** `bsp` contracts and types represent board resources/composition, not raw silicon abstractions duplicated from `mcu`.
* **Implementation location check:** concrete implementation structs stay out of `specs/` unless they are explicitly designated shared models.
* **Dependency direction check:** no reverse dependency is introduced against the `kernel -> specs`, `arch -> specs`, `bsp -> kernel + arch + mcu + specs` rule.
* **Unsafe boundary check:** `unsafe` usage remains confined to `arch/` and unavoidable startup code in `mcu/`.

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
    * Initialise the Cargo Workspace with a dedicated `bsp/` crate to act as the system "Matchmaker."
    * Define core Trait signatures in `specs/` for architecture, silicon hardware capabilities, and board-facing composition contracts.
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
    * Implement the Cortex-M `ContextSwitch` port in `arch/`.
    * Wire real PendSV save/restore handling into the startup vector path.
    * Implement manual task context initialisation and basic stack overflow detection.
    * Build a primitive switch demo and capture reproducible debug proof.
* **Deliverables:**
    * **Hardware Port**: ARM context-switch implementation with task frame setup and PendSV trigger path.
    * **Primitive Switch Evidence**: Debug-verified jump from `main()` into a prepared task context.
    * **Stack Guard Evidence**: Verified detection of intentional stack corruption via GDB.
* **Tests that must pass:** None (validation is via GDB and hardware; see Gatekeeper 2).
* **Gatekeeper 2:** A "Primitive Switch" demo where the CPU successfully jumps from `main()` to a single hard-coded function, verified via GDB register inspection.
* **Detailed implementation guide:** [docs/phase2/phase2_hardware_port.md](phase2/phase2_hardware_port.md).

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
* **Detailed implementation guide:** [docs/phase3/phase3_minimal_kernel.md](phase3/phase3_minimal_kernel.md).

---

## Phase 4: Safety-Critical Drivers (Concurrent with Phase 5)
**Objective:** Implement typestate-driven drivers as required for RC car control.

* **Tasks:**
    * Implement the STM32 GPIO driver with compile-time mode checking (required for motor/servo control).
    * Implement the STM32 UART driver in `drivers/uart` using the `Uart` trait (required for sensor/debug communication).
    * **Apply the Typestate Pattern**: Ensure methods like `write_byte()` and `set_high()` are inaccessible until the device `init()` has returned a valid state.
    * Add PWM driver with typestate for motor speed control (as needed).
* **Deliverables:**
    * **Typestate Driver API**: Drivers that use generic states (e.g., `Pin<Input>` vs `Pin<Output>`, `Uart<Uninitialized>` vs `Uart<Active>`) to restrict method availability at compile time.
    * **Error Handling**: Implementation of hardware-specific `Error` types associated with the `Uart`, `Gpio`, and `Pwm` traits.
    * **Compile-Fail Tests**: Verification that illegal hardware state access is rejected by the compiler.
* **Tests that must pass:** Compile-fail tests (e.g. using `trybuild`) verifying: (1) code that calls `write_byte()` before UART `init()` does not compile; (2) code that calls `set_high()` on an `Input` pin does not compile. Running `cargo test` passes when illegal usage is rejected.
* **Gatekeeper 4:** Test-case script proving the code fails to compile if driver setup sequences are violated.

---

## Phase 5: RC Car Integration & Real-World Validation (Expected: 5 weeks, Concurrent with Phase 4)
**Goal:** Demonstrate the ThetOS RTOS controlling a real mobile robot, validating practical applicability.

* **Tasks:**
    * **RC Car Hardware Integration**: Wire motor driver, servo controller, and sensor inputs to the STM32 board.
    * **Multi-Task Application**: Develop concurrent tasks for motor control, steering, sensor polling, and optional telemetry.
    * **Real-Time Validation**: Verify that the scheduler meets timing constraints under real-world load (motor acceleration, steering response).
    * **Demonstrate Typestate Safety**: Show that drivers enforce compile-time state machine guarantees in the RC car application.
* **Deliverables:**
    * **Functional RC Car**: A moving, steerable robot controlled by ThetOS.
    * **Multi-Task Application Code**: Example firmware demonstrating concurrent task execution in a real system.
    * **Video Demonstration**: Recorded evidence of the RC car operating autonomously or via remote control.
* **Tests that must pass:** Integration tests on physical hardware: (1) Motor accelerates/decelerates smoothly under task control; (2) Steering servo responds to commands with <X ms latency; (3) Multiple concurrent tasks (motor, steering, sensor) execute without race conditions or deadline misses.
* **Gatekeeper 5:** Functional RC car demonstrating that ThetOS can orchestrate real-time control of a complex embedded system with multiple concurrent tasks.

---

## Phase 6: Final Polish & Thesis Submission (Expected: Due date submission)
**Objective:** Finalise evidence, integrate RC car demonstration into thesis narrative, and complete submission.

* **Tasks:**
    * Finalise the `examples/` folder to demonstrate how a new board can be plugged in without changing kernel logic.
    * Document the RC car application as a case study demonstrating practical RTOS capabilities.
    * Write the 60-page thesis report integrating theory (static dispatch, typestate), implementation evidence, and RC car demonstration.
    * Final proofread, formatting compliance, and submission.
* **Final Deliverable:**
    * **Production Repository**: Fully commented, documented, and linted source code with RC car example.
    * **Thesis PDF**: The submitted technical document with RC car case study as proof-of-concept.
    * **RC Car Demonstration**: Video and/or live demonstration showing ThetOS-controlled RC car in operation.
* **Tests that must pass:** All tests from Phases 1, 3, 4, and 5 continue to pass; no regressions. RC car operates reliably in demonstration conditions.

---

## Summary of Milestones

| Milestone | Deliverable | Validation Method |
| :--- | :--- | :--- |
| **0** | Hardware Sanity & Panic Handler | LED Blink + Verified "Safe Halt" on code panic. |
| **1** | Build Pipeline & Host Unit Tests | `cargo test` success on x86 for kernel logic. |
| **2** | Primitive Switch & Stack Guard | Validated CPU register state + Verified Fault on Stack Overflow. |
| **3** | Scheduler & Critical Sections | Multi-blinky on hardware + Race-condition verification. |
| **4** | Typestate Drivers (Demand-Driven) | Compilation failure on attempted illegal hardware state access; GPIO/UART/PWM working in RC car application. |
| **5** | RC Car Integration & Real-Time Validation | Functional RC car demonstrating multi-task concurrent control and real-time responsiveness. |
| **6** | Thesis Submission with Case Study | 60-page thesis integrating theory, implementation, and RC car demonstration as proof-of-concept. |