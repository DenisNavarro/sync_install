#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

RUSTUP_VERSION=1.29.1
RUST_VERSION=1.99.0

main() {
    #ensure_vivaldi_is_installed
    #ensure_vscodium_and_its_extensions_are_installed
    install_apt_package_for_each_missing_executable vlc gimp
    apply_sync_install
    ensure_git_aliases
}

ensure_vivaldi_is_installed() {
    if ! command -v vivaldi >/dev/null 2>&1; then
        install_apt_package_if_executable_is_missing wget
        install_apt_package_if_executable_is_missing software-properties-common add-apt-repository
        ensure_apt_packages_are_installed ca-certificates gnupg
        # See https://doc.ubuntu-fr.org/vivaldi
        echo + Install Vivaldi
        wget -qO- https://repo.vivaldi.com/archive/linux_signing_key.pub | sudo apt-key add -
        run sudo add-apt-repository 'deb https://repo.vivaldi.com/archive/deb/ stable main'
        run sudo apt-get update
        run sudo apt-get install -y --no-install-recommends vivaldi-stable
    fi
}

ensure_vscodium_and_its_extensions_are_installed() {
    ensure_vscodium_is_installed
    for extension in timonwong.shellcheck rust-lang.rust-analyzer; do
        if ! codium --list-extensions | grep -Fxq "$extension"; then
            run codium --install-extension "$extension"
        fi
    done
}

ensure_vscodium_is_installed() {
    if ! command -v codium >/dev/null 2>&1; then
        install_apt_package_if_executable_is_missing wget
        ensure_apt_packages_are_installed ca-certificates gnupg
        # See https://vscodium.com/
        echo + Install VSCodium
        wget -qO - https://gitlab.com/paulcarroty/vscodium-deb-rpm-repo/raw/master/pub.gpg \
            | gpg --dearmor \
            | sudo dd of=/usr/share/keyrings/vscodium-archive-keyring.gpg
        echo -e 'Types: deb\nURIs: https://download.vscodium.com/debs\nSuites: vscodium\nComponents: main\nArchitectures: amd64 arm64\nSigned-by: /usr/share/keyrings/vscodium-archive-keyring.gpg' \
            | sudo tee /etc/apt/sources.list.d/vscodium.sources
        run sudo apt-get update
        run sudo apt-get install -y --no-install-recommends codium
    fi
}

apply_sync_install() {
    ensure_sync_install_is_installed
    if [[ ":$PATH:" != *":$HOME/.pixi/bin:"* ]]; then
        # Do this even if Pixi is not installed yet because `sync_install` may install it and call it.
        export PATH="$HOME/.pixi/bin:$PATH"
    fi
    if [ ! -f installed ]; then
        run touch installed
    fi
    run sync_install installed Dockerfile --go
    run cp Dockerfile installed
}

ensure_sync_install_is_installed() {
    ensure_cargo_is_available
    if ! command -v sync_install >/dev/null 2>&1; then
        install_apt_package_if_executable_is_missing gcc
        ensure_apt_packages_are_installed libc6-dev
        run cargo install --git https://github.com/DenisNavarro/sync_install --tag 0.12.0 --locked
    fi
}

ensure_cargo_is_available() {
    if ! command -v cargo >/dev/null 2>&1; then
        ensure_rust_is_installed
        if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
            export PATH="$PATH:$HOME/.cargo/bin"
        fi
    fi
    ensure_default_rust_toolchain "$RUST_VERSION"
}

ensure_rust_is_installed() {
    if [ ! -f ~/.cargo/bin/cargo ]; then
        install_apt_package_if_executable_is_missing wget
        ensure_apt_packages_are_installed ca-certificates
        # See https://github.com/rust-lang/docker-rust/blob/master/stable/bookworm/slim/Dockerfile
        run wget "https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/x86_64-unknown-linux-gnu/rustup-init"
        run chmod +x rustup-init
        run ./rustup-init -y --no-modify-path --default-toolchain "$RUST_VERSION"
        run rm rustup-init
    fi
}

ensure_default_rust_toolchain() {
    local toolchain="$1"
    ensure_has_rust_toolchain "$toolchain"
    if [[ "$(rustup show active-toolchain)" != "$toolchain"-* ]]; then
        run rustup default "$toolchain"
    fi
}

ensure_has_rust_toolchain() {
    local toolchain="$1"
    if ! has_rust_toolchain "$toolchain"; then
        run rustup toolchain install "$toolchain"
    fi
}

has_rust_toolchain() {
    local toolchain="$1"
    local line
    while IFS= read -r line; do
        [[ "$line" == "$toolchain"-* ]] && return 0
    done < <(rustup toolchain list)
    return 1
}

ensure_git_aliases() {
    if [ ! -f ~/.gitalias ]; then
        install_apt_package_if_executable_is_missing wget
        ensure_apt_packages_are_installed ca-certificates
        run wget https://raw.githubusercontent.com/GitAlias/gitalias/main/gitalias.txt -O ~/.gitalias
        git config set --global include.path ~/.gitalias
    fi
}

install_apt_package_for_each_missing_executable() {
    local pkg
    for pkg in "$@"; do
        install_apt_package_if_executable_is_missing "$pkg"
    done
}

install_apt_package_if_executable_is_missing() {
    local pkg="$1" exename="${2:-$1}"
    if ! command -v "$exename" >/dev/null 2>&1; then
        run sudo apt-get update
        run sudo apt-get install -y --no-install-recommends "$pkg"
    fi
}

ensure_apt_packages_are_installed() {
    local packages_to_install=()
    local pkg
    for pkg in "$@"; do
        local status rc=0
        status="$(dpkg-query -Wf='${db:Status-Status}' "$pkg")" || rc=$?
        if [ "$rc" -ne 0 ] || [ "$status" != installed ]; then
            packages_to_install+=("$pkg")
        fi
    done
    if [ ${#packages_to_install[@]} -ne 0 ]; then
        run sudo apt-get update
        run sudo apt-get install -y --no-install-recommends "${packages_to_install[@]}"
    fi
}

# Not used in the published `setup.bash` but useful to share
install_apt_package_if_gcc_cannot_include() {
    local pkg="$1" filename="$2"
    if ! echo "#include <$filename>" | gcc -E -x c - > /dev/null 2>&1; then
        run sudo apt-get update
        run sudo apt-get install -y --no-install-recommends "$pkg"
    fi
}

# Not used in the published `setup.bash` but useful to share with Ubuntu users
install_snap_package_if_executable_is_missing() {
    local pkg="$1" exename="${2:-$1}"
    if ! command -v "$exename" >/dev/null 2>&1; then
        run sudo snap install "$pkg"
    fi
}

# Not used in the published `setup.bash` but useful to share with Ubuntu users
install_classic_snap_package_if_executable_is_missing() {
    local pkg="$1" exename="${2:-$1}"
    if ! command -v "$exename" >/dev/null 2>&1; then
        run sudo snap install --classic "$pkg"
    fi
}

run() {
    printf '+ ' && print_shell_command "$@"
    "$@"
}

print_shell_command() {
    local result=()
    local arg
    for arg in "$@"; do
        result+=("$(printf %q "$arg")")
    done
    echo "${result[*]}"
}

main
