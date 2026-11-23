use std::env;
use std::path::Path;

fn main() {
    let dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let script = Path::new(&dir).join("kernel.ld");
    println!("cargo::rerun-if-changed=kernel.ld");
    println!("cargo::rustc-link-arg-bin=kernel=-T{}", script.display());
    println!("cargo::rustc-link-arg-bin=kernel=-Map=kernel.map");
}
