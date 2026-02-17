fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR not set");
        let icon_path = std::path::Path::new(&manifest_dir)
            .parent()
            .expect("failed to get parent directory")
            .parent()
            .expect("failed to get grandparent directory")
            .join("images/icon.ico");
        
        let mut res = winresource::WindowsResource::new();
        res.set_icon(icon_path.to_str().expect("invalid icon path"));
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to embed icon: {}", e);
        }
        
        println!("cargo:rerun-if-changed={}", icon_path.display());
    }
}
