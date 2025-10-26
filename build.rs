use chrono::Utc;
use std::process::Command;

fn main() {
    // Get git information
    let git_tag = get_git_output(&["git", "describe", "--tags", "--abbrev=0"])
        .unwrap_or_else(|| "unknown".to_string());
    let git_commit = get_git_output(&["git", "rev-parse", "--short", "HEAD"])
        .unwrap_or_else(|| "unknown".to_string());

    // Parse version from git tag (remove 'v' prefix if present)
    let version = git_tag.strip_prefix('v').unwrap_or(&git_tag);

    // Get current timestamp
    let build_date = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    // Set environment variables for build
    println!("cargo:rustc-env=VERSION={}", version);
    println!("cargo:rustc-env=GIT_TAG={}", git_tag);
    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit);
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Rerun if git changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");
    println!("cargo:rerun-if-changed=.git/packed-refs");
}

fn get_git_output(args: &[&str]) -> Option<String> {
    Command::new(args[0])
        .args(&args[1..])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}
