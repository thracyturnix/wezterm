fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Keep the displayed version to a simple three-digit build number.
    // The value advances with repository commits and wraps after 999.
    let ci_tag = std::process::Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|count| count.trim().parse::<u32>().ok())
        .map(|count| format!("{:03}", count % 1000))
        .unwrap_or_else(|| "000".to_string());

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=WEZTERM_TARGET_TRIPLE={}", target);
    println!("cargo:rustc-env=WEZTERM_CI_TAG={}", ci_tag);
}
