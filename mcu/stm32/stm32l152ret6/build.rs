//! Ships the STM32L152RE memory map to the linker.
//! Copies `memory.x` into OUT_DIR and adds it to the link search path, so any consumer
//! (in-tree or an external project depending on this crate) can link with `-Tmemory.x`
//! without knowing where this crate lives. Mirrors the cortex-m-rt `link.x` pattern.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    fs::copy("memory.x", out.join("memory.x")).expect("copy memory.x to OUT_DIR");
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");
}
