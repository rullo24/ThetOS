"""Shared helpers for build/flash/run/debug scripts (import from common.common or common)."""

from .common import (
    get_elf_path,
    get_gdb_instance,
    get_openocd_interface,
    get_openocd_target,
    get_target,
    repo_root,
)

__all__ = [
    "get_elf_path",
    "get_gdb_instance",
    "get_openocd_interface",
    "get_openocd_target",
    "get_target",
    "repo_root",
]
