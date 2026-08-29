# Gatekeeper 3 — evidence

Gatekeeper 3: two independent tasks toggling distinct GPIO pins, managed by the scheduler, on physical hardware, with no race conditions.

## Demonstration

The `double_gpio_toggle` package (under `phase_testing/phase3/`) spawns two equal-priority tasks, each owning its own output pin and toggling it on a timed delay. Under fixed-priority scheduling the two tasks round-robin. Build and flash per [`runbooks/phase3_gatekeeper.md`](runbooks/phase3_gatekeeper.md).

## Result

Flashed to the Nucleo-L152RE, both pins toggle continuously and independently under scheduler control, repeatable across power cycles.

## Why there are no race conditions

Each task owns a distinct pin object; there is no shared GPIO state. Pin writes go through the set/reset register — one bit per action, no read-modify-write — so a context switch between the two tasks' writes cannot corrupt either pin. Kernel scheduler state is mutated only inside a critical section, and the tick and context-switch exceptions run at equal priority so they do not nest destructively. Task stacks are disjoint and each carries a stack-guard canary.

## Host tests

The kernel host test suite (`cargo test -p kernel` on a host target) covers ready-list ordering, priority selection, preemption, blocking, and the tick path. It passes.

## Note

No photo, logic-analyser, or video capture was taken for this gatekeeper. The multi-task-on-hardware demonstration of record for the thesis is the RC car (thesis Phase 5), built as an external consumer of this release. The `double_gpio_toggle` binary plus this document are the Gatekeeper 3 evidence.
