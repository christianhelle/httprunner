use crate::colors;
use anyhow::Result;

pub fn run_upgrade() -> Result<()> {
    println!(
        "{} Upgrade is not supported on this platform",
        colors::red("❌")
    );
    Ok(())
}
