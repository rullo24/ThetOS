# ADR-0005: Blocking via a Blocked state and an idle task, not busy-wait

Status: Accepted

## Context
Tasks need to wait for a deadline. A task that spins on the tick counter while staying in the ready queue wastes CPU and, under fixed priority, starves everything below it.

## Decision
- `TaskState::Blocked` carries a `wake_at_tick`. `block_current_task_until()` leaves the ready queue; `on_tick_interrupt()` re-readies tasks whose deadline has passed. `delay_ms()` is a thin wrapper.
- A dedicated idle task (`TaskId(0)`, own 512 B stack, `wfi()` loop, lowest priority) is auto-spawned so something is always runnable when every task is blocked.

## Alternatives
- Busy-wait on a shared tick counter: no kernel change, but wastes cycles and breaks priority guarantees.
- No idle task, refuse to block the last runnable task: `delay_ms` then returns an error in the single-task case (the behaviour before the idle task existed).

## Consequences
- Lower-priority tasks get real windows while higher ones sleep.
- One reserved task id; `spawn_task` rejects `TaskId(0)`.
- One always-present stack (512 B), separate from the user pool.
