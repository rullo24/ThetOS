# ADR-0004: Fixed-priority preemptive scheduler

Status: Accepted (supersedes the earlier round-robin plan; docs realigned in #38)

## Context
The roadmap originally specified round-robin. Real-time control (the Phase 5 RC car) needs deadline-sensitive tasks to win over background work, which round-robin cannot guarantee.

## Decision
`FppScheduler`: one ready queue per priority level, lower `TaskPriority` value = higher priority (`MIN` 0, `MAX` 31, `DEFAULT` 15). `select_next_runnable` drains the highest non-empty queue; equal priority is FIFO (round-robin within a level). The policy sits behind the `SchedulerPolicy` trait and can be swapped.

## Alternatives
- Round-robin: simple, starvation-free, but no timing guarantees.
- EDF / dynamic priority: better utilisation, more state than the thesis needs now.

## Consequences
- Deterministic priority ordering; verified on hardware by `five_task_priority_demo`.
- A high-priority busy task can starve lower ones; mitigated by `delay_ms` (ADR-0005).
- `roadmap.md` and `phase3_minimal_kernel.md` reworded to fixed-priority preemptive (#38); "round-robin" now appears only where it means FIFO ordering within a single priority level.
