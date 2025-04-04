
.DELETE_ON_ERROR:
MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --warn-undefined-variables

.PHONY: debug # Debug execution
debug : test.maketarget
	cargo run -q -- dockerfiles/current_state_from_readme dockerfiles/target_state_from_readme

#############################################
# Other phony targets in alphabetical order #
#############################################

.PHONY: clean # Remove what is in `.gitignore`
clean :
	git clean -dXf

.PHONY: edit # Edit the `Makefile`
edit :
	@codium Makefile

.PHONY: help # Print each phony target with its description
help:
	@grep '^.PHONY: .* # ' Makefile | sed 's/\.PHONY: \(.*\) # \(.*\)/\1\t\2/' | expand -t 24

.PHONY: install_git_hooks # Install Git hooks with Cocogitto
install_git_hooks:
	cog install-hook --all --overwrite

.PHONY: install_rust_toolchains # Install the Rust toolchains used by the Git hooks
install_rust_toolchains:
	rustup toolchain install 1.85.0 --profile minimal
	rustup toolchain install 1.86.0 --profile minimal --component clippy,rustfmt

.PHONY: release # Release execution
release : test.maketarget
	cargo run -qr -- dockerfiles/current_state_from_readme dockerfiles/target_state_from_readme

################
# File targets #
################

fmt.maketarget : rustfmt.toml $(wildcard src/*.rs) $(wildcard tests/*.rs)
	cargo fmt && touch $@

clippy.maketarget : fmt.maketarget Cargo.toml
	cargo clippy --all-features --all-targets -- -D warnings && touch $@

test.maketarget : clippy.maketarget Cargo.lock
	cargo test && touch $@
