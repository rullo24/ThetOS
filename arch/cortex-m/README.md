# Cortex-M Architecture Support

This crate provides a modular implementation for the ARM Cortex-M family. It uses Rust **Feature Flags** to toggle hardware-specific capabilities at compile-time, ensuring zero-cost abstraction for different CPU variants.

## Sub-Directory Responsibilities
- `common/`: Shared logic for the Nested Vectored Interrupt Controller (NVIC) and the System Control Block (SCB).
- `v7m/`: Specifics for Cortex-M3/M4/M7.

## Feature Flags (Cargo.toml)
To support different "Pin Variants" and CPU capabilities without runtime checks, use:
- `fpu`: Enables Floating Point Unit register saving during context switches.

## Implementation Detail: The FPU Ghost
If the `fpu` feature is enabled, the `ContextSwitch` implementation must account for the extended stack frame (S0-S31 registers). Failure to enable this on FPU-active hardware will result in stack misalignment and memory corruption.