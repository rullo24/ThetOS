# Using ThetOS from an external project

A firmware project outside this workspace depends on ThetOS by git and builds a binary for its board. It depends on exactly two crates — the entry macro and the board support crate — and never names the internal contracts, kernel, architecture, or MCU crates directly.

The reference consumer is the RC-car firmware repository. Any package under `phase_testing/` is a working example of the application shape.

## What the consumer provides in its own files

**Manifest** — the two ThetOS crates by git ref, and `panic = "abort"` on every profile (profile settings do not propagate from a dependency):

```toml
[dependencies]
thetos_entry  = { git = "https://github.com/rullo24/ThetOS", branch = "main" }
nucleo_l152re = { git = "https://github.com/rullo24/ThetOS", branch = "main" }

[profile.dev]
panic = "abort"
[profile.release]
panic = "abort"
```

Pin a tag or revision instead of a branch for a reproducible build.

**`.cargo/config.toml`** — the target triple and two linker-script arguments:

```toml
[build]
target = "thumbv7m-none-eabi"

[target.thumbv7m-none-eabi]
rustflags = [
    "-C", "link-arg=-Tmemory.x",
    "-C", "link-arg=-Tcommon_minimal.ld",
]
```

Both linker scripts are copied into the build output directory and added to the link search path by ThetOS's MCU and architecture crates, so the bare `-T` names resolve with no path knowledge. The common script must come second.

**`src/main.rs`** — a `no_std`, `no_main` binary with a panic handler and an entry function annotated `#[entry(bsp = nucleo_l152re)]`. Naming the board crate in the attribute is what lets an external project skip the in-tree environment/build-script mechanism. Mirror any `phase_testing` demo for the full shape.

## Build and flash

Build with `cargo build`. Flash the resulting ELF with OpenOCD (ST-Link interface, STM32L1 target), pointing `-s` at your OpenOCD scripts directory. Flashing is done on the host, not inside a container.

`scripts/` in the ThetOS repo automate build/flash/debug for in-tree packages; an external project either copies them or calls OpenOCD directly.

## What the board crate re-exports

The board crate re-exports everything application code needs: the system handle and its task-control functions, the task identifier and priority types, and the full GPIO surface (pin handles, the pin type, direction markers, level and configuration enums, and the pin traits). Consult the board crate's `lib.rs` for the current list.
