
.DELETE_ON_ERROR:
MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --warn-undefined-variables

debug_exe_path = target/debug/sync_install
release_exe_path = target/release/sync_install

.PHONY: debug # Debug execution
debug : $(debug_exe_path) test.maketarget
	$< tests/current_state_from_readme tests/target_state_from_readme

#############################################
# Other phony targets in alphabetical order #
#############################################

.PHONY: clean # Remove what is in .gitignore
clean :
	git clean -dXf

.PHONY: edit # Edit the Makefile
edit :
	@codium Makefile

.PHONY: help # Print each phony target with its description
help:
	@grep '^.PHONY: .* # ' Makefile | sed 's/\.PHONY: \(.*\) # \(.*\)/\1\t\2/' | expand -t 18

.PHONY: install_git_hooks # Install Git hooks with Cocogitto
install_git_hooks:
	cog install-hook --all

.PHONY: release # Release execution
release : $(release_exe_path) test.maketarget
	$< tests/current_state_from_readme tests/target_state_from_readme

################
# File targets #
################

fmt.maketarget : rustfmt.toml $(wildcard src/*.rs) $(wildcard tests/*.rs)
	cargo fmt && touch $@

clippy.maketarget : fmt.maketarget Cargo.toml
	cargo clippy --all-features --all-targets -- -D warnings && touch $@

test.maketarget : clippy.maketarget
	cargo test && touch $@

$(debug_exe_path) : Cargo.toml Cargo.lock fmt.maketarget
	cargo build

$(release_exe_path) : Cargo.toml Cargo.lock fmt.maketarget
	cargo build --release
