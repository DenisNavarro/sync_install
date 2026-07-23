
`sync_install`
==============

`sync_install` is a CLI which updates what is installed by comparing two `Dockerfile`s.

I used it to update dependencies on my computer until July 2026.
Soon, in another repository, I will publish an idempotent script which does not depend on
`sync_install`.

## Installation

```bash
cargo install --git https://github.com/DenisNavarro/sync_install --tag 0.12.0 --locked
```

## Usage

For example, if the content of the `current_state` file is:

```Dockerfile
FROM docker.io/library/rust:1.97.1-slim-bookworm
RUN set -eux; \
    cargo install cargo-cache --version 0.8.3 --locked; \
    cargo install cocogitto --version 7.0.0 --locked; \
    cargo install fsays --version 0.3.0 --locked; \
    cargo cache -r all
CMD ["/bin/bash"]
```

and if the content of the `target_state` file is:

```Dockerfile
FROM docker.io/library/rust:1.97.1-slim-bookworm
RUN set -eux; \
    cargo install cargo-cache --version 0.8.3; \
    cargo install cocogitto --version 7.0.0 --locked; \
    cargo install pixi --git https://github.com/prefix-dev/pixi.git --tag v0.68.0 --locked; \
    cargo cache -r all
CMD ["/bin/bash"]
```

then the output of `sync_install current_state target_state` will be:

```
This is a dry run. Add the --go option to execute the below command(s).
---> [cargo uninstall fsays]
---> [cargo install cargo-cache --version 0.8.3 --force]
---> [cargo install pixi --git https://github.com/prefix-dev/pixi.git --tag v0.68.0 --locked]
```

Warning: `sync_install` is limited to a few use cases of its author and the
format of the `Dockerfile` contents must follow some arbitrary rules.

Tip: Update by comparing a `Dockerfile` to its state in the Git index:

```bash
sync_install <(git show :./Dockerfile) Dockerfile
sync_install <(git show :./Dockerfile) Dockerfile --go
```

If you wonder what features are implemented, you can look at
[the corresponding unit tests](./src/happy_path_tests.rs).

Then, if you wonder what are the arbitrary format rules, you can look at
[more unit tests](./src/parsing_error_tests.rs).

`sync_install` is used by [`setup.bash`](./idempotent_setup/setup.bash), an idempotent script to
install dependencies on a Debian-like OS.
See [`legacy_idempotent_setup`](./legacy_idempotent_setup) for more details.

## Remark

The unpublished previous version of this CLI read two YAML files instead of two `Dockerfile`s, but it
did not fit my use cases.

A `Dockerfile` is more flexible and can be used as a checkable documentation.
