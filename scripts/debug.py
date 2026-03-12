#!/usr/bin/env python3
"""Start OpenOCD in the background and launch GDB to debug the binary."""

import argparse
import subprocess
import sys
from pathlib import Path

# make scripts/common importable when run from repo root.
_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))
from common.common import (
    get_elf_path,
    get_gdb_instance,
    get_openocd_interface,
    get_openocd_target,
    repo_root,
)

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Start OpenOCD in the background and attach GDB to the board. Loads ELF, breaks at main, then continues.",
        epilog="Example: python3 scripts/debug.py -p no_rtos_blinky",
    )
    parser.add_argument(
        "-p", "--package",
        required=True,
        help="Cargo package name (binary to debug)",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help="Use release build",
    )
    args: argparse.Namespace = parser.parse_args()

    elf: Path = get_elf_path(args.package, args.release)
    if not elf.exists():
        print(f"ELF not found: {elf}", file=sys.stderr)
        print("Run scripts/build.py first.", file=sys.stderr)
        sys.exit(1)

    root = repo_root()
    # OpenOCD stays running to serve GDB on :3333 (no -c exit)
    openocd = subprocess.Popen(
        [
            "openocd",
            "-f", get_openocd_interface(),
            "-f", get_openocd_target(),
        ],
        cwd=root,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    try:
        # Connect, load ELF, break at main, continue; user can Ctrl+C in GDB to stop
        gdb_cmds = [
            "target extended-remote :3333",
            "load",
            "monitor reset halt",
            "break main",
            "continue",
        ]
        result = subprocess.run(
            [
                get_gdb_instance(),
                str(elf),
                "-ex", " ".join(gdb_cmds),
            ],
            cwd=root,
        )
        sys.exit(result.returncode)
    finally:
        # Tear down OpenOCD when GDB exits so we don't leave the daemon running
        openocd.terminate()
        openocd.wait()
