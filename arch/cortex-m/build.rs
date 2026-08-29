//! Ships the common Cortex-M section layout to the linker.
//! Copies `common_minimal.ld` into OUT_DIR and adds it to the link search path, so any
//! consumer (in-tree or an external project) can link with `-Tcommon_minimal.ld` after
//! `-Tmemory.x` without knowing where this crate lives. Mirrors the cortex-m-rt `link.x` pattern.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let script = "src/common/common_minimal.ld";
    fs::copy(script, out.join("common_minimal.ld")).expect("copy common_minimal.ld to OUT_DIR");
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed={script}");
    println!("cargo:rerun-if-changed=build.rs");
}
