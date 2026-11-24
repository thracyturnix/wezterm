pub fn wezterm_version() -> &'static str {
    // See build.rs
    env!("WEZTERM_CI_TAG")
}

pub fn wezterm_target_triple() -> &'static str {
    // See build.rs
    env!("WEZTERM_TARGET_TRIPLE")
}

/// The full commit count of this build. Unlike `wezterm_version`, it
/// does not wrap, so it can be compared against published builds.
pub fn wezterm_build_count() -> u64 {
    // See build.rs
    env!("WEZTERM_BUILD_COUNT").parse().unwrap_or(0)
}
