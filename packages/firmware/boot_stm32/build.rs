use std::{env, path::PathBuf};

fn main() {
    // Copy the linker script into our output directory and then pass
    // it to the linker.
    let input = PathBuf::from(env::var_os("LINKER_SCRIPT").expect("No linker script provided via LINKER_SCRIPT environment variable"));
    let linker_script = std::fs::read_to_string(input).expect("Failed to read linker script");
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    std::fs::write(out.join("memory.x"), linker_script).unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    if env::var("CARGO_FEATURE_DEFMT").is_ok() {
        println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    }
}
