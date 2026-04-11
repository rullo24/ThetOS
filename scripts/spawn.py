#!/usr/bin/env python3
"""Start OpenOCD w/ specific item"""

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
        description="Start OpenOCD in the background",
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
    try:
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
        print("OpenOCD started in background.", flush=True)
        print("""
Run (in GDB):
    "target extended-remote :3333",
    "load",
    "monitor reset halt",
    "break main",
    "continue",
        """)
        openocd.wait() # blocking until openocd exits

    finally:
        openocd.terminate()
        openocd.wait()

    # gdb_cmds = [
    #     "target extended-remote :3333",
    #     "load",
    #     "monitor reset halt",
    #     "break main",
    #     "continue",
    # ]

if __name__ == "__main__":
    main()
