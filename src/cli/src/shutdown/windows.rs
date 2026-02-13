use httprunner_core::colors;
// On Windows, we need to use both ctrlc and direct Windows Console API
// to properly handle CTRL+C in both cmd.exe and PowerShell
use windows_sys::Win32::Foundation::TRUE;
use windows_sys::Win32::System::Console::{SetConsoleCtrlHandler, CTRL_BREAK_EVENT, CTRL_C_EVENT};

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
        let result = SetConsoleCtrlHandler(Some(console_handler), TRUE);
        if result == 0 {
            // Registration failed; make this diagnosable without changing shutdown behavior
            eprintln!(
                "{} Failed to register Windows CTRL+C handler; PowerShell CTRL+C handling may not work as expected.",
                colors::yellow("⚠️"),
            );
        }
    }
}
