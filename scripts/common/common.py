"""Shared config and path helpers for build/flash/run/debug scripts."""

from pathlib import Path
import tomllib # type: ignore
from typing import Optional

# cached helper to get the repo root (for relative paths w/o cwd issues)
def repo_root() -> Path:
    # __file__ is scripts/common/common.py -> .parent.parent.parent = repo root
    root: Path = Path(__file__).resolve().parent.parent.parent
    if not (root / "Cargo.toml").is_file() or not (root / ".cargo" / "config.toml").is_file():
        raise SystemExit("Not in ThetOS repo: Cargo.toml or .cargo/config.toml not found")
    return root

# Path to the built ELF for the given package and profile (Cargo uses no .elf extension)
def get_elf_path(package_name: str, is_release: bool) -> Path:
    root: Path = repo_root()
    profile: str = "release" if is_release else "debug"
    target: str = get_target()
    return root / "target" / target / profile / package_name

# capture target from cargo config
def get_target() -> str:
    target: Optional[str] = CARGO_CONFIG.get("build", {}).get("target")
    if not target or target == "":
        raise SystemExit("No build target found in .cargo/config.toml")
    return target

# capture openocd interface from cargo config
def get_openocd_interface() -> str:
    v: Optional[str] = CARGO_CONFIG.get("scripting", {}).get("openocd_interface")
    return v or "interface/stlink.cfg"

# capture openocd target from cargo config
def get_openocd_target() -> str:
    v: Optional[str] = CARGO_CONFIG.get("scripting", {}).get("openocd_target")
    return v or "target/stm32l1.cfg"

def get_openocd_scripts_dir() -> str:
    v: Optional[str] = CARGO_CONFIG.get("scripting", {}).get("openocd_scripts_dir")
    return v or "/usr/share/openocd/scripts"

# capture gdb instance from cargo config
def get_gdb_instance() -> str:
    v: Optional[str] = CARGO_CONFIG.get("scripting", {}).get("gdb_instance")
    return v or "arm-none-eabi-gdb"

#########################
### PRIVATE FUNCTIONS ###
#########################

# helper func to pull config values from .cargo/config.toml for use in scripts
def _load_cargo_config() -> dict:
    path = repo_root() / ".cargo" / "config.toml"
    if not path.exists():
        return {}
    with open(path, "rb") as f:
        return tomllib.load(f)

# Cached cargo config (nested dict from TOML) for use in scripts
CARGO_CONFIG: dict = _load_cargo_config()