fn main() {
    tauri_build::build();
    for icon in [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/64x64.png",
        "icons/icon.icns",
        "icons/icon.ico",
        "icons/icon.png",
        "icons/icon.svg",
    ] {
        println!("cargo:rerun-if-changed={icon}");
    }
}
