#!/usr/bin/env python3
"""List OpenOCD interface and target config files (cross-platform)."""

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

# Resolved path to openocd executable if found in PATH, else None
def find_openocd_binary() -> Path | None:
    path = shutil.which("openocd")
    return Path(path).resolve() if path else None

# Derive OpenOCD scripts dir from binary path (prefix/share/openocd/scripts)
def scripts_dir_from_binary(openocd_bin: Path) -> Path:
    prefix = openocd_bin.resolve().parent.parent
    return prefix / "share" / "openocd" / "scripts"

# Run openocd -d3 and parse output for script search path; return if found
def scripts_dir_from_openocd_output() -> Path | None:
    result = subprocess.run(
        ["openocd", "-d3", "-c", "exit"],
        capture_output=True,
        text=True,
        timeout=5,
    )
    combined = (result.stderr or "") + (result.stdout or "")
    # Look for a line that contains a path and "script" (e.g. "script search directory: ...")
    for line in combined.splitlines():
        if "script" in line.lower():
            # Heuristic: take a path that contains "openocd" and "scripts"
            match = re.search(r"([^\s]+openocd[^\s]*scripts[^\s]*)", line)
            if match:
                p = Path(match.group(1).strip().strip('"').strip("'"))
                if (p / "target").is_dir():
                    return p
    return None

# OpenOCD scripts dir: env OPENOCD_SCRIPT_DIR, else binary-derived, else from openocd -d3
def get_openocd_scripts_dir() -> Path | None:
    if os.environ.get("OPENOCD_SCRIPT_DIR"):
        p = Path(os.environ["OPENOCD_SCRIPT_DIR"]).resolve()
        if (p / "target").is_dir():
            return p
    openocd_bin = find_openocd_binary()
    if not openocd_bin:
        return None
    derived = scripts_dir_from_binary(openocd_bin)
    if (derived / "target").is_dir():
        return derived
    return scripts_dir_from_openocd_output()

# Sorted list of .cfg stems (filename without extension) in the given directory
def list_cfg_stems(dir_path: Path) -> list[str]:
    if not dir_path.is_dir():
        return []
    return sorted(p.stem for p in dir_path.glob("*.cfg"))

# Parse args, find scripts dir, print requested interface/target config names
def main() -> None:
    parser = argparse.ArgumentParser(
        description="List OpenOCD interface and/or target config files. Uses OPENOCD_SCRIPT_DIR if set, else derives from openocd binary.",
        epilog="Example: python3 scripts/list_openocd.py --targets",
    )
    parser.add_argument(
        "--interfaces",
        action="store_true",
        help="List interface configs (e.g. stlink.cfg)",
    )
    parser.add_argument(
        "--targets",
        action="store_true",
        help="List target configs (e.g. stm32l1.cfg)",
    )
    args = parser.parse_args()

    # return both as a default
    if not args.interfaces and not args.targets:
        args.interfaces = True
        args.targets = True

    openocd_bin = find_openocd_binary()
    if not openocd_bin:
        print("openocd not found in PATH.", file=sys.stderr)
        sys.exit(1)

    scripts_dir = get_openocd_scripts_dir()
    if not scripts_dir:
        print("Could not find OpenOCD scripts directory (tried OPENOCD_SCRIPT_DIR and path derived from openocd binary).", file=sys.stderr)
        sys.exit(1)

    if args.interfaces:
        stems = list_cfg_stems(scripts_dir / "interface")
        print("Interfaces (use as interface/<name>.cfg):")
        for s in stems:
            print(f"  interface/{s}.cfg")
        if args.targets:
            print()

    if args.targets:
        stems = list_cfg_stems(scripts_dir / "target")
        print("Targets (use as target/<name>.cfg):")
        for s in stems:
            print(f"  target/{s}.cfg")


if __name__ == "__main__":
    main()
