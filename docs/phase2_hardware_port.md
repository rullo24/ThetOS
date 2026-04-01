# Phase 2 Implementation Guide: Hardware Port

## Goal
Implement the low-level context switch machinery so the CPU can move from `main()` into a prepared task context on hardware.

This phase builds unsafe architecture plumbing only. Scheduler policy and ready-list behaviour are Phase 3 concerns.

---

## Current repo baseline
Already present:
- Target and linker wiring in `.cargo/config.toml`.
- Common linker rules in `arch/cortex-m/common/common_minimal.ld`.
- Startup and vector table in `mcu/stm32/stm32l152ret6/src/startup.rs`.
- `ContextSwitch` trait contract in `specs/src/cpu/context_switch.rs`.
- Existing scripts in `scripts/build.py`, `scripts/flash.py`, and `scripts/debug.py`.

Missing for Phase 2 completion:
- Concrete ARM `ContextSwitch` implementation.
- Real PendSV handler wired into the vector table.
- Primitive switch demo and reproducible debug evidence.

---

## Step-by-step requirements

1. **Lock architecture assumptions**
- Fix the initial target assumptions (`thumbv7m-none-eabi`, Cortex-M profile, interrupt/stack model).
- Declare intended stack usage for Thread and Handler execution.
- Done when assumptions are stable and written down once.

2. **Create architecture implementation surface**
- Add implementation files under `arch/cortex-m/` for context setup and switch operations.
- Keep `kernel/` hardware-blind; wiring occurs at BSP/example composition level.
- Done when a concrete type can implement `specs::cpu::ContextSwitch`.

3. **Define canonical task context layout**
- Define the exact saved frame shape and ordering expected by your save/restore path.
- Ensure alignment constraints are explicit and consistent.
- Done when there is one authoritative layout document in `arch/cortex-m/README.md` or adjacent note.

4. **Implement `initialiseTaskContext(...)`**
- Build the initial stack frame from `stack_top`, `entry_point`, and `entry_arg`.
- Enforce pointer validity and deterministic initial frame values.
- Done when a new task context can be created reliably and invalid inputs fail safely.

5. **Implement `triggerPendSwitch()`**
- Implement a minimal PendSV request path.
- Keep logic lean and deterministic.
- Done when a trigger call sets PendSV pending as seen in debugger.

6. **Implement and wire PendSV handler**
- Write register save/restore core in assembly or `global_asm!`.
- Update `mcu/stm32/stm32l152ret6/src/startup.rs` so vector table entry points to your PendSV handler.
- Done when handler returns into prepared task context rather than `Default_Handler`.

7. **Add stack overflow detection**
- Implement a first-pass mechanism (watermark/canary, or MPU if chosen now).
- Define fail-safe behaviour for detection events.
- Done when forced stack corruption is detected and observable in debug.

8. **Create primitive switch demo**
- Add an example crate under `examples/phase2/` for one-task primitive switch proof.
- Demonstrate CPU handover from `main()` to a prepared task function.
- Done when task entry is reached reliably on hardware with visible proof (breakpoint/counter/GPIO).

9. **Write reproducible debug runbook**
- Add a short `docs/` procedure with exact build/flash/debug commands and expected pass/fail evidence.
- Use existing scripts where possible.
- Done when another developer can reproduce results without ad hoc steps.

---

## Practical acceptance checklist
- [ ] ARM `ContextSwitch` implementation exists and is consumable.
- [ ] `initialiseTaskContext(...)` creates valid initial task contexts.
- [ ] `triggerPendSwitch()` reliably requests PendSV.
- [ ] PendSV handler is wired into vector table and performs save/restore path.
- [ ] Stack guard mechanism detects intentional corruption.
- [ ] Primitive switch demo proves jump from `main()` to task context.
- [ ] Reproducible debug runbook exists with concrete success criteria.

---

## Gatekeeper 2 evidence package
Minimum evidence:
- Build artefact from Phase 2 example.
- Debugger capture/log notes showing PendSV path is taken.
- Register/PC evidence showing handover into task entry.
- Stack guard fault evidence from a forced corruption scenario.

Gatekeeper 2 passes when the primitive switch proof is repeatable on hardware and the overflow detection path is demonstrably active.
