
Legacy idempotent setup
=======================

`setup.bash` was my previous idempotent script to install dependencies on a Debian-like OS.

`setup.bash` installs APT packages, Rust, `sync_install` and other stuffs. It also uses
`sync_install` to install Rust crates and `conda-forge` recipes. Then, when you want to add,
remove or update Rust crates and `conda-forge` recipes, you can update the `Dockerfile` and launch
`make installed` or relaunch `setup.bash`.

`verify_dockerfile_and_setup.bash` uses Podman to build the `Dockerfile`
(to check its consistency) and to check `setup.bash` in a Debian image.

An updated version of this script is available in the [`idempotent_setup`][] repository.
This new version does not depend on `sync_install`.

[`idempotent_setup`]: https://github.com/DenisNavarro/idempotent_setup
