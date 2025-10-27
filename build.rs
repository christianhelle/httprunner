use chrono::Utc;
use std::process::Command;

fn main() {
    // Embed Windows icon
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("images/icon.ico");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to embed icon: {}", e);
        }
    }

    // Get Cargo.toml version as fallback
    let cargo_version = env!("CARGO_PKG_VERSION");

    // Get git information
    let git_tag = get_git_output(&["git", "describe", "--tags", "--abbrev=0"])
        .filter(|s| !s.is_empty() && s != "unknown");
    let git_commit =
        get_git_output(&["git", "rev-parse", "--short", "HEAD"]).filter(|s| !s.is_empty());

    // Determine version: prefer git tag, fallback to Cargo.toml version
    let version = if let Some(ref tag) = &git_tag {
        tag.strip_prefix('v').unwrap_or(tag).to_string()
    } else {
        cargo_version.to_string()
    };

    let git_tag_display = git_tag.unwrap_or_else(|| format!("v{}", cargo_version));
    let git_commit_display = git_commit.unwrap_or_else(|| "unknown".to_string());

    // Get current timestamp
    let build_date = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    // Set environment variables for build
    println!("cargo:rustc-env=VERSION={}", version);
    println!("cargo:rustc-env=GIT_TAG={}", git_tag_display);
    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit_display);
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
