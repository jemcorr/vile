fn main() {
    tauri_build::build();
    // `generate_context!` embeds this at compile time for the macOS dev icon.
    println!("cargo:rerun-if-changed=icons/icon.icns");
}
