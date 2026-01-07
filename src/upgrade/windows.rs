use crate::colors;
use crate::error::Result;
use std::process::Command;

pub fn run_upgrade() -> Result<()> {
    println!(
        "{} Upgrading httprunner to the latest version...",
        colors::blue("🚀")
    );

    let command = "irm https://christianhelle.com/httprunner/install.ps1 | iex";
    println!("{} Running: {}", colors::yellow("📦"), command);

    let output = Command::new("powershell.exe")
        .args(["-Command", command])
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
