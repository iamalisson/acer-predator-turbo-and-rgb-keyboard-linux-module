#!/bin/bash
# Installs the Predator RGB settings app and its restore-at-boot service.
#   ./install.sh          build and install (run as your normal user; asks for sudo when needed)
#   ./install.sh remove   uninstall (your settings in ~/.config/predator-rgb are kept)
set -euo pipefail
cd "$(dirname "$0")"

if [[ $EUID -eq 0 ]]; then
    echo "Run this as your normal user, not with sudo: it builds with your Rust toolchain and asks for sudo itself."
    exit 1
fi

if [[ ${1:-install} == remove ]]; then
    sudo systemctl disable --now predator-rgb.service 2>/dev/null || true
    sudo rm -f /usr/local/bin/predator-rgb /etc/systemd/system/predator-rgb.service \
        /usr/local/share/applications/predator-rgb.desktop
    sudo systemctl daemon-reload
    echo "Removed. Your settings are still in ~/.config/predator-rgb/."
    exit 0
fi

if ! command -v cargo >/dev/null; then
    echo "cargo not found. Install Rust first: https://rustup.rs (Arch: sudo pacman -S rust)"
    exit 1
fi

cargo build --release
sudo install -Dm755 target/release/predator-rgb /usr/local/bin/predator-rgb
sudo install -Dm644 predator-rgb.desktop /usr/local/share/applications/predator-rgb.desktop

config="${XDG_CONFIG_HOME:-$HOME/.config}/predator-rgb/config.json"
sed "s|@CONFIG@|$config|" predator-rgb.service | sudo tee /etc/systemd/system/predator-rgb.service >/dev/null
sudo systemctl daemon-reload
sudo systemctl enable predator-rgb.service

echo "Installed. Open \"Predator RGB\" from your app menu, or run: predator-rgb"
