mod args;
mod banner;

pub use args::Cli;
pub use banner::show_donation_banner;

#[cfg(test)]
mod tests;
