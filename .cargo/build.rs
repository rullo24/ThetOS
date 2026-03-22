use std::fs;
use std::path::PathBuf;

/// DESCRIPTION
/// Cargo runs this before compiling `no_rtos_basic` -> cargo: lines are instructions to Cargo
fn workspace_root() -> PathBuf {
    if let Ok(dir) = std::env::var("CARGO_WORKSPACE_DIR") {
        return PathBuf::from(dir);
    }
    let mut dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.is_file() {
            let Ok(src) = fs::read_to_string(&cargo_toml) else {
                if !dir.pop() {
                    panic!("could not find workspace root");
                }
                continue;
            };
            if src.lines().any(|l| l.trim_start().starts_with("[workspace]")) {
                return dir;
            }
        }
        if !dir.pop() {
            panic!("could not find workspace root; use a recent Cargo or open the repo as a workspace");
        }
    }
}

/// DESCRIPTION
/// finds config.toml and extracts the bsp crate name -> injecting THETOS_BSP for #[entry] proc macro to read
fn main() {
    let cfg_path = workspace_root().join(".cargo").join("config.toml");
    let raw = fs::read_to_string(&cfg_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", cfg_path.display()));
    let v: toml::Value = toml::from_str(&raw).expect("parse .cargo/config.toml");
    let bsp = v
        .get("thetos")
        .and_then(|t| t.get("bsp"))
        .and_then(|x| x.as_str())
        .expect(".cargo/config.toml must contain [thetos] bsp = \"crate_name\"");
    println!("cargo:rustc-env=THETOS_BSP={bsp}");
    println!("cargo:rerun-if-changed={}", cfg_path.display());
}