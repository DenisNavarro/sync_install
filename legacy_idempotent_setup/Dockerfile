FROM docker.io/library/rust:1.98.0-slim-bookworm

RUN set -eux; \
    cargo install cargo-cache --version 0.8.3; \
    cargo install cocogitto --version 7.0.0 --locked; \
    cargo install pixi --git https://github.com/prefix-dev/pixi --tag v0.77.0 --locked; \
    cargo cache -r all

WORKDIR /work
COPY pixi.toml pixi.lock /work/

RUN set -eux; \
    pixi run -e make cargo install uv --git https://github.com/astral-sh/uv --tag 0.12.5 --locked; \
    cargo cache -r all

RUN set -eux; \
    pixi global install git=2.55.0; \
    pixi clean cache --yes

ENV HOME="/root"
ENV PATH="$HOME/.pixi/bin:$PATH"

RUN set -eux; \
    git config set --global user.email your.email@address.com; \
    git config set --global user.name 'Your name'; \
    git config set --global core.editor nano; \
    git config set --global init.defaultBranch main; \
    cat "$HOME/.gitconfig"
