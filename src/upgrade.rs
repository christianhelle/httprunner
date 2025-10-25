use crate::colors;
use anyhow::Result;
use std::process::Command;

#[cfg(target_os = "windows")]
pub fn run_upgrade() -> Result<()> {
    println!(
        "{} Upgrading httprunner to the latest version...",
        colors::blue("🚀")
    );

    let command = "irm https://christianhelle.com/httprunner/install.ps1 | iex";
    println!("{} Running: {}", colors::yellow("📦"), command);

    let output = Command::new("powershell.exe")
        .args(&["-Command", command])
        .output()?;

    if output.status.success() {
        println!("{} Upgrade completed successfully!", colors::green("✅"));
    } else {
        println!(
            "{} Upgrade failed with exit code: {:?}",
            colors::red("❌"),
            output.status.code()
        );
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn run_upgrade() -> Result<()> {
    println!(
        "{} Upgrading httprunner to the latest version...",
        colors::blue("🚀")
    );

    // Check if installed via snap
    let is_snap = Command::new("snap")
        .args(&["list", "httprunner"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let (shell, args): (&str, Vec<&str>) = if is_snap {
        ("snap", vec!["refresh", "httprunner"])
    } else {
        let command = "curl -fsSL https://christianhelle.com/httprunner/install | bash";
        println!("{} Running: {}", colors::yellow("📦"), command);
        ("/bin/bash", vec!["-c", command])
    };

    let output = Command::new(shell).args(&args).output()?;

    if output.status.success() {
        println!("{} Upgrade completed successfully!", colors::green("✅"));
    } else {
        println!(
            "{} Upgrade failed with exit code: {:?}",
            colors::red("❌"),
            output.status.code()
        );
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn run_upgrade() -> Result<()> {
    println!(
        "{} Upgrading httprunner to the latest version...",
        colors::blue("🚀")
    );

    let command = "curl -fsSL https://christianhelle.com/httprunner/install | bash";
    println!("{} Running: {}", colors::yellow("📦"), command);

    let output = Command::new("/bin/bash").args(&["-c", command]).output()?;

    if output.status.success() {
        println!("{} Upgrade completed successfully!", colors::green("✅"));
    } else {
        println!(
            "{} Upgrade failed with exit code: {:?}",
            colors::red("❌"),
            output.status.code()
        );
    }

    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn run_upgrade() -> Result<()> {
    println!(
        "{} Upgrade is not supported on this platform",
        colors::red("❌")
    );
    Ok(())
}
