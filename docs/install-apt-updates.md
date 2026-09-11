# Updates through APT

Debian packages from this build register the signed repository automatically.
After installing one of those packages, normal system updates will include
new WezTerm builds:

```console
sudo apt update
sudo apt upgrade
```

For a machine that already has an older standalone package, register the
repository once:

```console
sudo install -d -m 0755 /usr/share/keyrings
curl -fsSL https://thracyturnix.github.io/wezterm/thracyturnix-wezterm-archive-keyring.gpg | sudo tee /usr/share/keyrings/thracyturnix-wezterm-archive-keyring.gpg >/dev/null
echo 'deb [signed-by=/usr/share/keyrings/thracyturnix-wezterm-archive-keyring.gpg] https://thracyturnix.github.io/wezterm stable main' | sudo tee /etc/apt/sources.list.d/thracyturnix-wezterm.list >/dev/null
sudo apt update
sudo apt install wezterm
```
