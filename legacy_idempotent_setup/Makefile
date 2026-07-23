MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --warn-undefined-variables

.PHONY: dry_run
dry_run :
	sync_install installed Dockerfile

.PHONY: clean
clean :
	rm -rf .pixi

installed : Dockerfile
	[ -f $@ ] || touch -- $@
	sync_install $@ $< --go
	cp -- $< $@
