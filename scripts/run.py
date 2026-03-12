#!/usr/bin/env python3
"""Build and flash the binary."""

import argparse
import subprocess
import sys
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))  # cwd and script paths work when run from repo root


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Build the package (unless --no-build) then flash it to the board.",
        epilog="Example: python3 scripts/run.py -p no_rtos_blinky",
    )
    parser.add_argument(
        "-p", "--package",
        required=True,
        help="Cargo package to build and flash",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help="Build and flash release binary",
    )
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="Skip build; flash existing binary only",
    )
    args = parser.parse_args()

    root = _SCRIPTS_DIR.parent  # repo root so subprocess cwd is correct
    build_py = _SCRIPTS_DIR / "build.py"
    flash_py = _SCRIPTS_DIR / "flash.py"

    if not args.no_build:
        # Invoke via subprocess so we don't import and run build/flash mains on import
        build_cmd = [sys.executable, str(build_py), "-p", args.package]
        if args.release:
            build_cmd.append("--release")
        result = subprocess.run(build_cmd, cwd=root)
        if result.returncode != 0:
            sys.exit(result.returncode)

    flash_cmd = [sys.executable, str(flash_py), "-p", args.package]
    if args.release:
        flash_cmd.append("--release")
    result = subprocess.run(flash_cmd, cwd=root)
    sys.exit(result.returncode)


if __name__ == "__main__":
    main()
