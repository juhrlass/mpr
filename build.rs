use embed_manifest::{embed_manifest, new_manifest};
use std::process::Command;

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest(new_manifest("mpr")).expect("unable to embed manifest file");
    }
    
    // Nach der Kompilierung: Icon mit rcedit setzen
    // rcedit muss im PATH verfügbar sein
    println!("cargo:warning=Versuche Icon mit rcedit zu setzen...");
    
    let rcedit_result = Command::new("rcedit")
        .args(&["target\\release\\mpr.exe", "--set-icon", "logo.ico"])
        .output();
    
    match rcedit_result {
        Ok(output) => {
            println!("cargo:warning=rcedit Exit-Code: {}", output.status);
            println!("cargo:warning=rcedit stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("cargo:warning=rcedit stderr: {}", String::from_utf8_lossy(&output.stderr));
            
            if output.status.success() {
                println!("cargo:warning=Icon erfolgreich mit rcedit gesetzt!");
            } else {
                println!("cargo:warning=rcedit Fehler: Exit-Code {}", output.status);
                println!("cargo:warning=Kein Icon wird angehängt!");
            }
        }
        Err(e) => {
            println!("cargo:warning=rcedit nicht gefunden im PATH: {}", e);
            println!("cargo:warning=Kein Icon wird angehängt!");
        }
    }
    
    println!("cargo:rerun-if-changed=build.rs");
}
