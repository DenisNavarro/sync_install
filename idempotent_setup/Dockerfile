FROM docker.io/library/rust:1.95.0-slim-bookworm

RUN set -eux; \
    cargo install amber --version 0.6.1 --locked; \
    cargo install bat --version 0.26.1 --locked; \
    cargo install cargo-cache --version 0.8.3; \
    cargo install chars --version 0.7.0 --locked; \
    cargo install du-dust --version 1.2.4 --locked; \
    cargo install dua-cli --version 2.34.0 --locked; \
    cargo install eza --version 0.23.4 --locked; \
    cargo install fend --version 1.5.8 --locked; \
    cargo install fish --git https://github.com/fish-shell/fish-shell --tag 4.7.1 --locked; \
    cargo install fselect --version 0.10.0 --locked; \
    cargo install pixi --git https://github.com/prefix-dev/pixi --tag v0.68.0 --locked; \
    cargo install ripgrep --version 15.1.0 --locked; \
    cargo install rnr --version 0.5.1 --locked; \
    cargo install ruplacer --version 0.10.0 --locked; \
    cargo install rusti-cal --version 1.1.0 --locked; \
    cargo install sd --version 1.0.0 --locked; \
    cargo install skim --version 4.6.2 --locked; \
    cargo install tealdeer --version 1.8.1 --locked; \
    cargo cache -r all

WORKDIR /work
COPY pixi.toml pixi.lock /work/

RUN set -eux; \
    pixi run -e openssl-pkgconfig cargo install cargo-update --version 20.0.0 --locked; \
    # Alternative:
    #pixi run -e make-perl cargo install cargo-update --version 20.0.0 --features vendored-openssl --locked; \
    pixi run -e make cargo install fd-find --version 10.4.2 --locked; \
    pixi run -e openssl-pkgconfig cargo install typst-cli --version 0.14.2 --locked; \
    pixi run -e make cargo install uv --git https://github.com/astral-sh/uv --tag 0.11.13 --locked; \
    cargo cache -r all

RUN set -eux; \
    pixi global install curl=8.20.0; \
    pixi global install ffmpeg=8.0.1; \
    pixi global install git=2.54.0; \
    pixi global install make=4.3; \
    pixi global install rsync=3.4.1; \
    pixi clean cache --yes

ENV HOME="/root"
ENV PATH="$HOME/.pixi/bin:$PATH"

RUN set -eux; \
    git config set --global user.email your.email@address.com; \
    git config set --global user.name 'Your name'; \
    git config set --global core.editor nano; \
    git config set --global init.defaultBranch main; \
    cat "$HOME/.gitconfig"
