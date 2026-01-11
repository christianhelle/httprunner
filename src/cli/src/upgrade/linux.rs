use crate::colors;
use anyhow::Result;
use std::process::Command;

pub fn run_upgrade() -> Result<()> {
    println!(
        "{} Upgrading httprunner to the latest version...",
        colors::blue("🚀")
    );

    // Check if installed via snap
    let is_snap = Command::new("snap")
        .args(["list", "httprunner"])
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

    let output = Command::new(shell).args(args).output()?;

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
