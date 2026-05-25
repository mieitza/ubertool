COVERAGE_MIN ?= 75

.PHONY: help build release test fmt clippy gen verify-spec coverage coverage-summary coverage-gate ci clean

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

coverage:  ## generate HTML coverage report at target/llvm-cov/html/
	cargo llvm-cov clean --workspace && cargo llvm-cov --lib --tests --html

coverage-summary:  ## print line/function/region coverage to stdout
	cargo llvm-cov clean --workspace && cargo llvm-cov --lib --tests --summary-only

coverage-gate:  ## enforce minimum line coverage (used in CI)
	cargo llvm-cov clean --workspace && cargo llvm-cov --lib --tests --fail-under-lines $(COVERAGE_MIN)

ci: fmt clippy test verify-spec  ## what CI runs locally

fuzz-smoke:  ## run each fuzz target for 30s (CI smoke check)
	@for t in fuzz_json fuzz_yaml fuzz_toml fuzz_xml fuzz_csv; do \
	    echo "==> fuzz $$t (30s)"; \
	    PATH="$$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin:$$PATH" cargo fuzz run $$t -- -max_total_time=30 || exit 1; \
	done

clean:  ## remove build artifacts and generated docs
	cargo clean
	rm -rf docs/cli docs/llms.txt
