use crate::colors;
use anyhow::Result;
use std::process::Command;

pub fn run_upgrade() -> Result<()> {
    println!(
        "{} Upgrading httprunner to the latest version...",
        colors::blue("🚀")
    );

    let command = "curl -fsSL https://christianhelle.com/httprunner/install | bash";
    println!("{} Running: {}", colors::yellow("📦"), command);

    let output = Command::new("/bin/bash").args(["-c", command]).output()?;

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
