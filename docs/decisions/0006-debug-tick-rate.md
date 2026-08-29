# ADR-0006: 100 Hz SysTick on debug builds

Status: Accepted

## Context
Bug #41: tasks stopped advancing. Root cause was timing, not logic. A debug (unoptimised) `on_tick_interrupt()` measured ~7200 cycles. At the MSI clock then in use (2.097 MHz) a 1 ms tick budgeted only ~2096 cycles, so the handler could not finish before the next tick and starved thread-mode code. Release builds measured ~430 cycles and were fine.

## Decision
`SYSTICK_PERIOD_MS = 10` (100 Hz). Reload = `SYSCLK_HZ * 10 / 1000 - 1`; with MSI later raised to 4.194 MHz that is 41942, widening the margin further. The DWT cycle counter and `TICK_CYCLES_LAST` / `TICK_CYCLES_MAX` are kept permanently as a budget guard.

## Alternatives
- Keep 1 kHz and only run release builds: loses on-target debugging.
- Optimise the tick handler to fit 1 ms in debug: fragile, fights the compiler.

## Consequences
- Timer-driven delays have 10 ms granularity on debug builds.
- `TICK_CYCLES_MAX` must stay well under the reload; approaching it is a regression.
- Do not lower the tick period on a debug build without re-measuring.
