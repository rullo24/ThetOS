# Architecture Abstraction Layer (Arch)

This directory contains the CPU-specific implementations of the traits defined in `specs/`. It is responsible for the "Unsafe" core of the RTOS, including context switching, interrupt handling, and register-level CPU configuration.

## Directory Structure
> To be extended in future.
- `cortex-m/`: Implementations for the ARM Cortex-M family.

## The Arch Contract
Every architecture crate must implement the following from `specs/`:
1. **`ContextSwitch`**: Manual save/restore of CPU registers.
2. **`SystemTimer`**: Logic for the SysTick or equivalent heartbeat.

## Design Philosophy: "One Step Lower"
Architecture is split into **Generic** logic (e.g., NVIC is similar across most Cortex-M) and **Instruction Set** logic (e.g., FPU handling and BasePri differ between M0, M4, and M33).