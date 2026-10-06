# Repository notes

- On LMDE, `lsb_release` identifies the host as Linuxmint, while WezTerm packages use the Debian base version. `ci/deploy.sh` must map LMDE to Debian packaging and fail with a nonzero exit status if a platform is unsupported. Never treat a successful exit with no package as a successful build.
- When the user has authorized building and installing WezTerm, that authorization includes fixing this repository's LMDE packaging path and retrying it. Complete the build, installation, and installed-binary verification without asking again for the same approval.
