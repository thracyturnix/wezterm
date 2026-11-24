fn git(args: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Re-run whenever a commit is made or the checked-out commit changes,
    // so the build number never goes stale in incremental builds.
    if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        let git_dir = std::path::PathBuf::from(git_dir);
        println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
        println!(
            "cargo:rerun-if-changed={}",
            git_dir.join("packed-refs").display()
        );
        if let Some(head_ref) = git(&["symbolic-ref", "-q", "HEAD"]) {
            println!(
                "cargo:rerun-if-changed={}",
                git_dir.join(head_ref).display()
            );
        }
    }

    // Keep the displayed version to a simple three-digit build number.
    // The value advances with repository commits and wraps after 999.
    let count = git(&["rev-list", "--count", "HEAD"])
        .and_then(|count| count.parse::<u64>().ok())
        .unwrap_or(0);
    let ci_tag = format!("{:03}", count % 1000);

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=WEZTERM_TARGET_TRIPLE={}", target);
    println!("cargo:rustc-env=WEZTERM_CI_TAG={}", ci_tag);
    // Full, non-wrapping commit count used by the update checker.
    println!("cargo:rustc-env=WEZTERM_BUILD_COUNT={}", count);
}
