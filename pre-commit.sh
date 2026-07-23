#!/bin/sh
set -ex

cd "$(git rev-parse --show-toplevel)"

cargo +1.85.1 test --locked --workspace
cargo +1.97.1 fmt --all --check
cargo +1.97.1 clippy --all-features --all-targets --locked --workspace -- -D warnings
cargo +1.97.1 test --locked --workspace

if ! git diff --cached --quiet -- dockerfiles; then (
    cd dockerfiles
    podman build -f current_state_from_readme -t sync_install_current_state_from_readme
    podman build -f target_state_from_readme -t sync_install_target_state_from_readme
    podman build -f tested_example_1 -t sync_install_tested_example_1
    podman build -f tested_example_2 -t sync_install_tested_example_2
    podman image prune -f
) fi

if ! git diff --cached --quiet -- legacy_idempotent_setup; then
    bash legacy_idempotent_setup/verify_dockerfile_and_setup.bash
fi
