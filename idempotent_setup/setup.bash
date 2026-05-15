#!/usr/bin/env bash
set -xeuo pipefail

main() {
    #ensure_vivaldi_is_installed
    #ensure_vscodium_and_its_extensions_are_installed
    install_apt_package_if_executable_is_missing vlc
    apply_sync_install
    ensure_git_aliases
}

ensure_vivaldi_is_installed() {
    if ! command -v vivaldi >/dev/null 2>&1; then
        install_apt_package_if_executable_is_missing wget
        install_apt_package_if_executable_is_missing software-properties-common add-apt-repository
        ensure_apt_package_is_installed ca-certificates
        ensure_apt_package_is_installed gnupg
        # See https://doc.ubuntu-fr.org/vivaldi
        wget -qO- https://repo.vivaldi.com/archive/linux_signing_key.pub | sudo apt-key add -
        sudo add-apt-repository 'deb https://repo.vivaldi.com/archive/deb/ stable main'
        sudo apt-get update
        sudo apt-get install -y --no-install-recommends vivaldi-stable
    fi
}

ensure_vscodium_and_its_extensions_are_installed() {
    ensure_vscodium_is_installed
    for dep in timonwong.shellcheck rust-lang.rust-analyzer; do
        if ! codium --list-extensions | grep -Fxq "$dep"; then
            codium --install-extension "$dep"
        fi
    done
}

ensure_vscodium_is_installed() {
    if ! command -v codium >/dev/null 2>&1; then
        install_apt_package_if_executable_is_missing wget
        ensure_apt_package_is_installed ca-certificates
        ensure_apt_package_is_installed gnupg
        # See https://vscodium.com/
        wget -qO - https://gitlab.com/paulcarroty/vscodium-deb-rpm-repo/raw/master/pub.gpg \
            | gpg --dearmor \
            | sudo dd of=/usr/share/keyrings/vscodium-archive-keyring.gpg
        echo -e 'Types: deb\nURIs: https://download.vscodium.com/debs\nSuites: vscodium\nComponents: main\nArchitectures: amd64 arm64\nSigned-by: /usr/share/keyrings/vscodium-archive-keyring.gpg' \
            | sudo tee /etc/apt/sources.list.d/vscodium.sources
        sudo apt-get update
        sudo apt-get install -y --no-install-recommends codium
    fi
}

apply_sync_install() {
    ensure_sync_install_is_installed
    if [[ ":$PATH:" != *":$HOME/.pixi/bin:"* ]]; then
        # Do this even if Pixi is not installed yet because `sync_install` may install it and call it.
        export PATH="$HOME/.pixi/bin:$PATH"
    fi
    if [ ! -f installed ]; then
        touch installed
    fi
    sync_install installed Dockerfile --go
    cp Dockerfile installed
}

ensure_sync_install_is_installed() {
    ensure_rust_is_installed
    if ! command -v sync_install >/dev/null 2>&1; then
        install_apt_package_if_executable_is_missing gcc
        ensure_apt_package_is_installed libc6-dev
        cargo install --git https://github.com/DenisNavarro/sync_install --tag 0.11.1 --locked
    fi
}

ensure_rust_is_installed() {
    if ! command -v cargo >/dev/null 2>&1; then
        if [ ! -f ~/.cargo/bin/cargo ]; then
            install_apt_package_if_executable_is_missing wget
            ensure_apt_package_is_installed ca-certificates
            # See https://github.com/rust-lang/docker-rust/blob/master/stable/bookworm/slim/Dockerfile
            wget https://static.rust-lang.org/rustup/archive/1.29.0/x86_64-unknown-linux-gnu/rustup-init
            chmod +x rustup-init
            ./rustup-init -y --no-modify-path
            rm rustup-init
        fi
        if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
            export PATH="$PATH:$HOME/.cargo/bin"
        fi
    fi
}

ensure_git_aliases() {
    if [ ! -f ~/.gitalias ]; then
        install_apt_package_if_executable_is_missing wget
        wget https://raw.githubusercontent.com/GitAlias/gitalias/main/gitalias.txt -O ~/.gitalias
        git config set --global include.path ~/.gitalias
    fi
}

ensure_apt_package_is_installed() {
    status="$(dpkg-query -Wf='${db:Status-Status}' "$1")" || rc=$?
    rc=${rc:-0}
    if [ "$rc" -ne 0 ] || [ "$status" != installed ]; then
        sudo apt-get update
        sudo apt-get install -y --no-install-recommends "$1"
    fi
}

install_apt_package_if_executable_is_missing() {
    local exename="${2:-$1}"
    if ! command -v "$exename" >/dev/null 2>&1; then
        sudo apt-get update
        sudo apt-get install -y --no-install-recommends "$1"
    fi
}

install_apt_package_if_gcc_cannot_include() {
    if ! echo "#include <$2>" | gcc -E -x c - > /dev/null 2>&1; then
        sudo apt-get update
        sudo apt-get install -y --no-install-recommends "$1"
    fi
}

# For Ubuntu
install_snap_package_if_executable_is_missing() {
    local exename="${2:-$1}"
    if ! command -v "$exename" >/dev/null 2>&1; then
        sudo snap install "$1"
    fi
}

# For Ubuntu
install_classic_snap_package_if_executable_is_missing() {
    local exename="${2:-$1}"
    if ! command -v "$exename" >/dev/null 2>&1; then
        sudo snap install --classic "$1"
    fi
}

main
