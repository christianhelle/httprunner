use crate::colors;

pub fn run_upgrade() -> Result<()> {
    println!(
        "{} Upgrade is not supported on this platform",
        colors::red("❌")
    );
    Ok(())
}
