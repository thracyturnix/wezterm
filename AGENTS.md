Every build must produce a Debian package that can be installed with `apt`; a successful binary-only build is not complete.

# Repository notes

- On LMDE, `lsb_release` identifies the host as Linuxmint, while WezTerm packages use the Debian base version. `ci/deploy.sh` must map LMDE to Debian packaging and fail with a nonzero exit status if a platform is unsupported. Never treat a successful exit with no package as a successful build.
- The displayed build number is `git rev-list --count HEAD` mod 1000 (`wezterm-version/build.rs`). Commit changes before building so every build gets a new number, and after installing confirm `wezterm --version` matches the HEAD build number. `ci/deploy.sh` refuses to package a stale number; never bypass that check.
- When the user has authorized building and installing WezTerm, that authorization includes fixing this repository's LMDE packaging path and retrying it. Complete the build, installation, and installed-binary verification without asking again for the same approval.
