#!/usr/bin/env python3
"""Build embedded binary: rustup target add, then cargo build (linker from .cargo/config.toml)."""

import argparse
import subprocess
import sys
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))
from common.common import get_target, repo_root


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Build the embedded binary. Target from .cargo/config.toml. With no -p or --workspace, builds default-members only.",
        epilog="Examples: python3 scripts/build.py | python3 scripts/build.py -p no_rtos_basic | python3 scripts/build.py --workspace",
    )
    parser.add_argument(
        "-p",
        "--package",
        help="Build only this Cargo package",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help="Build in release mode",
    )
    parser.add_argument(
        "--workspace",
        action="store_true",
        help="Build entire workspace (all members)",
    )
    args = parser.parse_args()
    root = repo_root()

    target: str = get_target()
    subprocess.run(["rustup", "target", "add", target], check=True, cwd=root)

    cmd = ["cargo", "build", "--target", target]
    if args.workspace:
        cmd.append("--workspace")
        print("Building workspace...", flush=True)
    elif args.package:
        cmd.extend(["-p", args.package])
        print(f"Building '{args.package}'...", flush=True)
    else:
        print("Building default members...", flush=True)

    if args.release:
        cmd.append("--release")

    result = subprocess.run(cmd, cwd=root)
    if result.returncode == 0:
        print("Build succeeded.", flush=True)
    sys.exit(result.returncode)


if __name__ == "__main__":
    main()
