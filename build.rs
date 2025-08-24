use embed_manifest::{embed_manifest, new_manifest};
use std::process::Command;

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest(new_manifest("mpr")).expect("unable to embed manifest file");
    }
   
    println!("cargo:rerun-if-changed=build.rs");
}
