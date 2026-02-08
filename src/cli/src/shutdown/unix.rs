#[cfg(not(windows))]
pub fn enable_forceful_shutdown() {
    ctrlc::set_handler(|| {
        use std::io::{self, Write};
        let _ = writeln!(
            io::stderr(),
            "\n{} Received CTRL+C, shutting down...",
            colors::yellow("⚠️")
        );
        let _ = io::stderr().flush();
        std::process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");
}
