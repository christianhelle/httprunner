use httprunner_core::colors;
// On Windows, we need to use both ctrlc and direct Windows Console API
// to properly handle CTRL+C in both cmd.exe and PowerShell
use windows_sys::Win32::Foundation::TRUE;
use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler};

#[cfg(windows)]
pub fn enable_forceful_shutdown() {
    unsafe extern "system" fn console_handler(ctrl_type: u32) -> i32 {
        match ctrl_type {
            CTRL_C_EVENT | CTRL_BREAK_EVENT => {
                use std::io::{self, Write};
                let _ = writeln!(
                    io::stderr(),
                    "\n{} Received CTRL+C, shutting down...",
                    colors::yellow("⚠️")
                );
                let _ = io::stderr().flush();
                std::process::exit(130);
            }
            _ => 0, // Let default handler process other events
        }
    }

    unsafe {
        // Register Windows console control handler for PowerShell compatibility
        SetConsoleCtrlHandler(Some(console_handler), TRUE);
    }
}
