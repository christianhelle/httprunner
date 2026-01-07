use crate::colors;
use crate::error::Result;

pub fn run_upgrade() -> Result<()> {
    println!(
        "{} Upgrade is not supported on this platform",
        colors::red("❌")
    );
    Ok(())
}
