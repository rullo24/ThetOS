# Phase 3 Implementation Guide: Minimal Kernel & Scheduler

## Goal

Orchestrate **multiple tasks** through **hardware-blind** kernel logic: a **ready list**, **scheduler policy**, **atomic access** to kernel structures, and a **time-driven** path to request scheduling—without putting scheduling policy inside `arch/`.

Phase 2 established **how** the CPU switches contexts (PendSV, frames, stack guard). Phase 3 establishes **who runs next** and **when** preemption or yield is considered, while keeping `kernel/` free of register-level code.

---

## Current repo baseline

Already present:

- **`specs::kernel::CriticalSection`** — closure-based trait (`with_execute`) in `specs/src/kernel/critical_section.rs`.
- **`CortexMCriticalSection`** — masks/restores interrupts via `PRIMASK` in `arch/cortex-m/src/common/critical_section.rs`.
- **`Kernel<CtxSwitch, Crit, Scheduler, StackGuard>`** — generic kernel in `kernel/src/lib.rs` with spawn path, stack pool, stack guard slots, `yield_now` hooking PendSV.
- **`TaskControlBlock`** — `kernel/src/tcb.rs` implementing `specs::kernel::CoreTcb` with stack bounds, state, arch context, stack-guard context.
- **`FppScheduler`** — placeholder in `kernel/src/scheduler/fpp.rs` (`on_task_spawn` TODO); exported as `kernel::FppScheduler`.
- **`SchedulerPolicy`** — minimal trait in `specs/src/kernel/scheduler_policy.rs` (currently only `on_task_spawn`).
- **BSP wiring** — e.g. `bsp/nucleo/nucleo_l152re` composes `Kernel` with `V7mContextSwitch`, `CortexMCriticalSection`, `FppScheduler`, `CortexMStackGuard`.
- **Host tests** — `kernel/tests/test_host_kernel.rs` with mocks; roadmap expects expanded coverage in Phase 3.

Missing for Phase 3 completion:

- **Real ready-list semantics** (add / remove / select-next) protected by `CriticalSection`.
- **Scheduler policy** aligned with thesis roadmap (**round-robin** unless you explicitly choose otherwise and update the roadmap).
- **`SystemTimer` (or equivalent) trait** in `specs/` and a **board/arch** implementation driving periodic **tick → scheduler**.
- **End-to-end hardware demo**: **Gatekeeper 3** — two tasks, distinct GPIO (or equivalent observable outputs), no races (see evidence section).

---

## Implementation lock-ins

### Phase 3 locked assumptions

- **Kernel purity:** no PAC/register code in `kernel/`; timer and GPIO only through traits or BSP-owned glue.
- **Atomicity model:** kernel data structures updated only inside `CriticalSection::with_execute` (or equivalent) on targets where preemptive interrupts exist.
- **Dependency direction:** unchanged vs [docs/roadmap.md](../roadmap.md) architecture boundaries.
- **Scheduler location:** policy and ready list live in `kernel/`; `arch/` supplies `ContextSwitch`, `CriticalSection` impl, and interrupt/timer mechanics as declared by traits.
- **Naming note:** the tree currently contains **`FppScheduler`**. The roadmap calls for a **round-robin** scheduler; either **implement round-robin** (new type or evolve the scheduler module) or **update the roadmap** to fixed-priority if that is the deliberate thesis choice—do not leave the mismatch unowned.

---

## Step-by-step requirements

1. **Formalise the ready list**
   - Define an internal structure (queue, ring, etc.) holding `TaskId` (or handles) for **ready** tasks.
   - All mutations go through **`CriticalSection`**.
   - Done when unit tests can add/remove/select without mocks violating ordering assumptions.

2. **Expand `SchedulerPolicy` (or split traits) as needed**
   - Beyond `on_task_spawn`, add operations required for round-robin: e.g. **enqueue**, **dequeue next**, **notify ready**—exact surface is a design choice but must be **testable** on the host.
   - Done when the trait expresses the policy the kernel calls from `yield_now` / tick path.

3. **Implement round-robin (or approved policy) in `kernel/`**
   - Wire `on_task_spawn`, selection on yield/tick, and interaction with **`curr_task`** / `Kernel` state.
   - Done when host tests show deterministic ordering for a known sequence of spawns and yields (mock time if needed).

4. **Introduce `SystemTimer` (or `KernelTick`) in `specs/`**
   - Trait methods: e.g. **initialise**, **clear pending**, **handler hooks**—minimal surface to “one tick fired.”
   - Done when BSP or `arch` can implement it for STM32L1 SysTick (or chosen peripheral) without pulling kernel into `arch/` specifics.

5. **Implement timer + ISR path**
   - In **`mcu/`** or **`bsp/`**: configure timer, ISR calls into a **thin** callback that **only** requests scheduling (e.g. `Kernel::tick()` or sets a flag consumed in PendSV path—keep design consistent with your thesis).
   - Must not starve PendSV or violate priority rules documented for Cortex-M3.

6. **Connect tick to scheduler**
   - From timer context, safely signal the kernel (often “PendSV pending” or queue work); **never** run full scheduler inside ISR if it conflicts with `CriticalSection` rules—document the chosen model.
   - Done when a **tick** can cause a **different** ready task to run on hardware.

7. **Hardware demo (Gatekeeper 3)**
   - Two tasks, **distinct** GPIO pins (or LEDs), toggling under scheduler control.
   - Done when both pins show activity and GDB/logs show **no obvious data races** on kernel globals (validate with design + optional instrumentation).

8. **Tests (roadmap-mandated)**
   - `cargo test -p kernel` on a **host** target: ready-list tests with **mock** `ContextSwitch` and **mock** `CriticalSection`; tests that **scheduler** invokes **`trigger_pendsv_switch`** (or the trait method you standardise on) when policy says switch.
   - Done when CI/local `cargo test` satisfies the Phase 3 bullet in [docs/roadmap.md](../roadmap.md).

9. **Runbook / evidence (mirror Phase 2 style)**
   - Add **`docs/runbooks/phase3_gatekeeper.md`** (or extend a single runbooks index): build/flash/debug, **multi-blinky** pass/fail, optional logic-analyser notes for Gatekeeper 3.

---

## Practical acceptance checklist

- [ ] Ready list exists and is only mutated under `CriticalSection`.
- [ ] Scheduler policy is implemented and covered by host unit tests.
- [ ] `SystemTimer` (or equivalent tick trait) is defined in `specs/` and implemented for the target board.
- [ ] Timer interrupt path integrates with kernel without violating layer boundaries.
- [ ] `cargo test -p kernel` passes on host with tests enumerated in the roadmap.
- [ ] Gatekeeper 3 hardware demo: two tasks, two observables, reproducible procedure documented.

---

## Gatekeeper 3 evidence package

Minimum evidence:

- Binary from Phase 3 demo (workspace package under `phase_testing/` or agreed `examples/` layout).
- **Observation** of two independent outputs (GPIO/LED) toggling under scheduler control.
- Short note on **why** races are absent (critical sections + PendSV ordering)—one paragraph is enough for the thesis appendix.
- Optional: timestamped photo or logic-analyser capture for vivas.

Gatekeeper 3 passes when **multi-blinky** (or equivalent) is **repeatable** on hardware and host tests for the scheduler **remain green**.

---

## Related

- [docs/roadmap.md](../roadmap.md) — Phase 3 objectives, tests, Gatekeeper 3.
- [docs/phase2/phase2_hardware_port.md](../phase2/phase2_hardware_port.md) — context switch prerequisites.
