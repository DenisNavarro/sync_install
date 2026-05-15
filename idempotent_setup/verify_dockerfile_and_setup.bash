#!/usr/bin/env bash
set -xeuo pipefail

podman build -t dockerfile_next_to_setup .

# Check idempotency of `setup.bash` without `sudo`
podman build -f - -t setup . <<'EOF'
FROM debian:bookworm-slim
WORKDIR /work
COPY Dockerfile pixi.toml pixi.lock setup.bash /work/
RUN set -eux; \
    sed 's/sudo //g' setup.bash > setup_without_sudo.bash; \
    bash setup_without_sudo.bash; \
    bash setup_without_sudo.bash
ENV HOME="/root"
ENV PATH="$HOME/.pixi/bin:$PATH:$HOME/.cargo/bin"
RUN set -eux; \
    bash setup_without_sudo.bash; \
    cargo install --list; \
    pixi global list
EOF

podman image prune -f
