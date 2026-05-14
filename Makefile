.PHONY: help build release test fmt clippy gen verify-spec ci clean

help:  ## list targets
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) | awk -F':.*##' '{printf "  %-15s %s\n", $$1, $$2}'

build:  ## debug build
	cargo build

release:  ## release build (LTO, opt-z, strip)
	cargo build --release

test:  ## run all tests
	cargo test

fmt:  ## format
	cargo fmt --all

clippy:  ## lint
	cargo clippy --all-targets -- -D warnings

gen:  ## validate spec + regenerate docs/cli and docs/llms.txt
	ocli spec check ubertool.ocs.yaml
	mkdir -p docs/cli
	ocli gen docs --spec-file ubertool.ocs.yaml --output-dir docs/cli --format markdown --dryrun=false
	cat docs/cli/*.md > docs/llms.txt
	@echo "docs/cli/ and docs/llms.txt regenerated"

verify-spec: build  ## check spec ↔ CLI consistency
	./scripts/verify-spec.sh

ci: fmt clippy test verify-spec  ## what CI runs locally

clean:  ## remove build artifacts and generated docs
	cargo clean
	rm -rf docs/cli docs/llms.txt
