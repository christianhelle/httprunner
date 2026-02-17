use std::process::Command;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let icon_path = std::path::Path::new(&manifest_dir)
            .parent()
            .expect("failed to get parent directory")
            .parent()
            .expect("failed to get grandparent directory")
            .join("images/icon.ico");

        let mut res = winresource::WindowsResource::new();
        res.set_icon(icon_path.to_str().expect("invalid icon path"));
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to embed icon: {}", e);
        }

        println!("cargo:rerun-if-changed={}", icon_path.display());
    }

    let cargo_version = env!("CARGO_PKG_VERSION");

    let git_tag = get_git_output(&["git", "describe", "--tags", "--abbrev=0"])
        .filter(|s| !s.is_empty() && s != "unknown");
    let git_commit =
        get_git_output(&["git", "rev-parse", "--short", "HEAD"]).filter(|s| !s.is_empty());

    let version = if let Some(tag) = &git_tag {
        tag.strip_prefix('v').unwrap_or(tag).to_string()
    } else {
        cargo_version.to_string()
    };

    let git_tag_display = git_tag.unwrap_or_else(|| format!("v{}", cargo_version));
    let git_commit_display = git_commit.unwrap_or_else(|| "unknown".to_string());

    let build_date = {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System time before UNIX EPOCH");

        let secs = now.as_secs();
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        let (year, month, day) = days_to_ymd(days);

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            year, month, day, hours, minutes, seconds
        )
    };

    println!("cargo:rustc-env=VERSION={}", version);
    println!("cargo:rustc-env=GIT_TAG={}", git_tag_display);
    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit_display);
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

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

// Convert days since UNIX epoch to (year, month, day)
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let mut year = 1970;
    let mut remaining_days = days;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &days_in_month in &days_in_months {
        if remaining_days < days_in_month as u64 {
            break;
        }
        remaining_days -= days_in_month as u64;
        month += 1;
    }

    (year, month, remaining_days + 1)
}

fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}
