#!/usr/bin/env python3
"""Flash the built binary to the board via OpenOCD (ST-Link + STM32L1)."""

import argparse
import subprocess
import sys
from pathlib import Path

# make scripts/common importable when run from repo root.
_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))
from common.common import get_elf_path, get_openocd_interface, get_openocd_target, repo_root

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Flash the built ELF to the board via OpenOCD. Interface and target come from .cargo/config.toml [scripting].",
        epilog="Example: python3 scripts/flash.py -p no_rtos_blinky",
    )
    parser.add_argument(
        "-p", "--package",
        required=True,
        help="Cargo package name (binary to flash)",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help="Use release build",
    )
    args = parser.parse_args()

    elf: Path = get_elf_path(args.package, args.release)
    if not elf.exists():
        print(f"ELF not found: {elf}", file=sys.stderr)
        print("Run scripts/build.py first.", file=sys.stderr)
        sys.exit(1)

    root: Path = repo_root()
    print(f"Flashing {elf.name}...", flush=True)

    cmd = [
        "openocd",
        "-f", get_openocd_interface(),
        "-f", get_openocd_target(),
        "-c", f"program {elf} verify reset exit", # program = flash, verify = checksum, reset = run MCU, exit = quit OpenOCD
    ]
    result = subprocess.run(cmd, cwd=root)
    if result.returncode == 0:
        print("Flash succeeded.", flush=True)
    sys.exit(result.returncode)

if __name__ == "__main__":
    main()
