# Setup — from a clean machine to a blinking LED

## 1. Rust toolchain

Install **rustup** — <https://rustup.rs>. Do not use a distro-packaged `rustc`; the project relies on rustup reading `rust-toolchain.toml`.

That is all that is required for the Rust side. On the first `cargo` command in the repo, rustup reads `rust-toolchain.toml` and installs the pinned compiler version, the `thumbv7m-none-eabi` target, and the listed components automatically.

To run the kernel host tests you also need a host target, which is normally your machine's default and already present. If `cargo test -p kernel --target <host-triple>` complains it is missing:

```
rustup target add <host-triple>     # e.g. x86_64-unknown-linux-gnu, aarch64-apple-darwin, x86_64-pc-windows-msvc
```

Optional, for `cargo objdump` / `cargo size` on the ELF:

```
cargo install cargo-binutils
```

## 2. Flash / debug tooling (only if you will touch hardware)

You need three things:

| Tool | Purpose | Minimum |
|------|---------|---------|
| **OpenOCD** | talks to the ST-Link, programs flash | 0.11+ (STM32L1 support) |
| **arm-none-eabi GDB** | source-level debugging (`scripts/debug.py`) | any recent build; `gdb-multiarch` also works |
| **Python** | runs `scripts/*.py` | 3.11+ |

### macOS (Homebrew)

```
brew install openocd python
brew install --cask gcc-arm-embedded      # provides arm-none-eabi-gdb
```

### Debian / Ubuntu

```
sudo apt install openocd gdb-multiarch python3 stlink-tools
```

`stlink-tools` also installs the udev rules that let a non-root user access the probe. If your GDB is `gdb-multiarch`, set `gdb_instance = "gdb-multiarch"` in `.cargo/config.toml` `[scripting]`.

### Fedora

```
sudo dnf install openocd stlink arm-none-eabi-gdb python3
```

### Windows

Install OpenOCD and an `arm-none-eabi` GDB (scoop, choco, or the Arm GNU Toolchain installer), plus Python 3.11+. The ST-Link uses the WinUSB / ST-Link USB driver.

### Linux — probe permissions

If flashing fails with a permissions error, install your distro's `stlink` / `stlink-tools` package (it ships udev rules) or drop ST's `*-stlink*.rules` into `/etc/udev/rules.d/`, then:

```
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Unplug and replug the board.

## 3. Point the config at your OpenOCD install

`.cargo/config.toml` `[scripting]` has `openocd_scripts_dir`. Set it to wherever your OpenOCD keeps its `interface/` and `target/` scripts:

| Platform | Typical path |
|----------|--------------|
| Linux | `/usr/share/openocd/scripts` |
| macOS Homebrew (Apple Silicon) | `/opt/homebrew/share/openocd/scripts` |
| macOS Homebrew (Intel) | `/usr/local/share/openocd/scripts` |

`openocd_interface` (`interface/stlink.cfg`) and `openocd_target` (`target/stm32l1.cfg`) are correct for this board and do not normally need changing. `scripts/list_openocd.py` prints what your install provides.

## 4. Verify

```
cargo build --workspace
cargo test -p kernel --target <host-triple>
python3 scripts/run.py -p blinking_leds
```

The last command builds and flashes `blinking_leds`; the green user LED (LD2) should blink at roughly 1 Hz. If it does, the setup is complete.

## 5. Common problems

- **`can't find crate for 'test'` / missing `#[panic_handler]` when running `cargo test`** — you ran it without `--target <host-triple>`. The default target is embedded and has no test harness.
- **OpenOCD: `Can't find interface/stlink.cfg`** — `openocd_scripts_dir` is wrong for your machine (step 3).
- **OpenOCD: `Error: open failed` / `LIBUSB_ERROR_ACCESS`** — probe permissions; Linux udev rules (step 2).
- **`rust-toolchain.toml` ignored** — you are not using rustup, or an env var is pinning a toolchain.
