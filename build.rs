use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let source = Path::new("req/config.ini");
    let dest = Path::new(&out_dir).join("config.ini");
    fs::copy(source, dest).expect("Failed to copy config!");
    println!("cargo:rerun-if-changed=req/config.toml");
}