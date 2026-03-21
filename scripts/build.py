#!/usr/bin/env python3
"""Build embedded binary. Ensures target is installed, then runs cargo build."""

import os
import argparse
import subprocess
import sys
import re
from pathlib import Path

# make scripts/common importable when run from repo root.
_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))
from common.common import get_target, repo_root

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Build the embedded binary. Target from .cargo/config.toml. With no -p or --workspace, builds default-members only.",
        epilog="Examples: python3 scripts/build.py  (default-members) | python3 scripts/build.py -p no_rtos_blinky | python3 scripts/build.py --workspace",
    )
    parser.add_argument(
        "-p", "--package",
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

    # Idempotent: no-op if target already installed
    target: str = get_target()
    profile: str = "release" if args.release else "debug"
    bsp_memory_x_src = root / "boards" / "nucleo" / "nucleo-l152re" / "memory.x"
    if not bsp_memory_x_src.is_file():
        raise SystemExit(f"BSP memory.x not found: {bsp_memory_x_src}")

    def symlink_memory_x_into_cortex_m_rt_out_dirs(out_dirs: set[Path]) -> None:
        for out_dir in out_dirs:
            memory_x_dst = out_dir / "memory.x"
            memory_x_dst.parent.mkdir(parents=True, exist_ok=True)
            if memory_x_dst.exists() or memory_x_dst.is_symlink():
                memory_x_dst.unlink()
            os.symlink(bsp_memory_x_src.resolve(), memory_x_dst)

    def parse_memory_x_missing_out_dirs(build_output: str) -> set[Path]:
        # Extract the generated `.../cortex-m-rt-<hash>/out/link.x` path and place `memory.x`
        # alongside it, since `link.x` contains `INCLUDE memory.x`.
        out_dirs: set[Path] = set()
        pattern = re.compile(r'(/[^:\s]+/cortex-m-rt-[^/\s]+/out)/link\.x')
        for m in pattern.finditer(build_output):
            out_dirs.add(Path(m.group(1)))
        return out_dirs

    def looks_like_memory_x_missing(build_output: str) -> bool:
        return "cannot find linker script memory.x" in build_output or "INCLUDE memory.x" in build_output

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

    # Try once, then fix `memory.x` in cortex-m-rt's generated out dir(s) and retry.
    for attempt in range(2):
        result = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
        combined = (result.stdout or "") + (result.stderr or "")
        if combined:
            print(combined, flush=True)
        if result.returncode == 0:
            print("Build succeeded.", flush=True)
            sys.exit(0)

        if attempt == 0 and args.package and looks_like_memory_x_missing(combined):
            build_out_root = root / "target" / target / profile / "build"
            out_dirs = set(build_out_root.glob("cortex-m-rt-*/out"))
            if not out_dirs:
                out_dirs = parse_memory_x_missing_out_dirs(combined)
            if out_dirs:
                symlink_memory_x_into_cortex_m_rt_out_dirs(out_dirs)
                continue

        sys.exit(result.returncode)

if __name__ == "__main__":
    main()
