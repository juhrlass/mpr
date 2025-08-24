use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest(new_manifest("mpr")).expect("unable to embed manifest file");

        let mut res = winres::WindowsResource::new();
    // Hier den Pfad zu deiner ICO-Datei angeben
    res.set_icon("assets/logo.ico");
    if let Err(e) = res.compile() {
        eprintln!("Fehler beim Kompilieren der Windows-Ressourcen: {}", e);
    }

    }
    println!("cargo:rerun-if-changed=build.rs");
}
