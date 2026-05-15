
Idempotent setup
================

`setup.bash` is an idempotent script to install dependencies. You may be interested in downloading
this script and adapting it to your needs when you get a new computer with a Debian-like OS.

You can also download the whole `idempotent_setup` directory if you want to keep the call to
`sync_install` in `setup.bash`.

`setup.bash` installs APT packages, Rust, `sync_install` and other stuffs. It also uses
`sync_install` to install Rust crates and `conda-forge` recipes. Then, when you want to add,
remove or update Rust crates and `conda-forge` recipes, you can update the `Dockerfile` and launch
`make installed` or relaunch `setup.bash`.

`verify_dockerfile_and_setup.bash` uses Podman to build the `Dockerfile`
(to check its consistency) and to check `setup.bash` in a Debian image.
