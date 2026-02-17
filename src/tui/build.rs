fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../images/icon.ico");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to embed icon: {}", e);
        }
    }
}
