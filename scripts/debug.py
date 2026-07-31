#!/usr/bin/env python3
"""Start OpenOCD in the background and launch GDB to debug the binary."""

import argparse
import subprocess
import sys
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))
from common.common import (
    get_elf_path,
    get_gdb_instance,
    get_openocd_interface,
    get_openocd_target,
    get_openocd_scripts_dir,
    repo_root,
)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Start OpenOCD in the background and attach GDB to the board. Loads ELF, breaks at main, then continues.",
        epilog="Example: python3 scripts/debug.py -p no_rtos_basic",
    )
    parser.add_argument(
        "-p",
        "--package",
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
    scripts_dir = get_openocd_scripts_dir()
    print("Starting OpenOCD in background...", flush=True)
    openocd = subprocess.Popen(
        [
            "openocd",
            "-s",
            scripts_dir,
            "-f",
            get_openocd_interface(),
            "-f",
            get_openocd_target(),
        ],
        cwd=root,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    try:
        gdb_cmds = [
            "target extended-remote :3333",
            "load",
            "monitor reset halt",
            "monitor cortex_m maskisr steponly",  # keep interrupts live on `continue`, hold them off only mid-step
            "break main",
            "continue",
        ]

        gdb_instance = get_gdb_instance()
        if subprocess.run(["which", gdb_instance], capture_output=True, text=True).returncode != 0:
            print(f"GDB instance '{gdb_instance}' not found", file=sys.stderr)
            print("Install gdb-multiarch or arm-none-eabi-gdb for your distro.", file=sys.stderr)
            sys.exit(1)

        print(f"Launching GDB for {elf.name} (break at main, then continue)...", flush=True)
        gdb_call = [get_gdb_instance(), str(elf)]
        for cmd in gdb_cmds:
            gdb_call.extend(["-ex", cmd])
        result = subprocess.run(gdb_call, cwd=root)
        print(f"GDB session ended (exit code {result.returncode}).", flush=True)
        sys.exit(result.returncode)
    finally:
        print("Stopping OpenOCD...", flush=True)
        openocd.terminate()
        openocd.wait()


if __name__ == "__main__":
    main()
