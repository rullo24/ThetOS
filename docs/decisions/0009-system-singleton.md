# ADR-0009: System is a global singleton reached via free functions

Status: Accepted

## Context
`System::run()` moves the value into a `static`, because the SysTick ISR and kernel must reach it. Tasks run after that move. A `&System` taken in `app_main` would dangle across the move, and two tasks holding `&mut System` would alias.

## Decision
There is one `System`. Task code calls module free functions (`system::delay_ms`, `system::current_tick`, `system::yield_now`) that reach the stored instance. `app_main` holds the value only long enough to spawn tasks and call `run()`.

## Alternatives
- Return `&'static mut System` from construction and pass it in: needs an `init()`-into-static lifecycle change plus an `unsafe` re-borrow to give each task a shared ref. Ceremony around a value that is global anyway.
- Per-task `Sys` handle (ZST): typed sugar over the same global.

## Consequences
- Task code names `system::` rather than an argument. Standard RTOS idiom (FreeRTOS `vTaskDelay`, embassy time driver).
- No compile-time proof of init; not a real failure mode (single mandatory singleton, created before any task runs).
- Revisit only if multiple `System` instances are ever needed.
