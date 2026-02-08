#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::enable_forceful_shutdown;

#[cfg(not(target_os = "windows"))]
mod unix;

#[cfg(not(target_os = "windows"))]
pub use unix::enable_forceful_shutdown;
