# Fuzzing ubertool parsers

Five libFuzzer targets live in `fuzz/fuzz_targets/`, one per format parser (json, yaml, toml, xml, csv). They feed arbitrary bytes to each parser and fail only on a **panic** — any `Result::Err` is acceptable behavior.

## Prerequisites

```bash
cargo install cargo-fuzz
rustup toolchain install nightly
```

The project's `rust-toolchain.toml` pins the main crate to stable. The `fuzz/rust-toolchain.toml` overrides this to nightly for the fuzz sub-workspace, but because the system `cargo` may be a non-rustup binary (e.g., from Homebrew), you may need to prefix commands with the nightly toolchain's bin directory:

```bash
export PATH="$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin:$PATH"
```

Replace `aarch64-apple-darwin` with your host triple (`rustup show active-toolchain` to find it).

## Running a single target

```bash
# 10-minute run for fuzz_json
cargo fuzz run fuzz_json -- -max_total_time=600

# Available targets:
#   fuzz_json   fuzz_yaml   fuzz_toml   fuzz_xml   fuzz_csv
```

## Smoke run (all targets, 30 s each)

```bash
make fuzz-smoke
```

This is intentionally not wired into `make ci` — it requires nightly and takes ~3 minutes.

## Where files live

| Path | Contents |
|---|---|
| `fuzz/corpus/<target>/` | Saved interesting inputs (gitignored) |
| `fuzz/artifacts/<target>/crash-*` | Reproducer for any crash found (gitignored) |

## When a crash is found

cargo-fuzz writes the reproducing input to `fuzz/artifacts/<target>/crash-<hash>`.

1. Reproduce: `cargo fuzz run <target> fuzz/artifacts/<target>/crash-<hash>`
2. Read the panic message and locate the source line.
3. Fix the bug (replace `unwrap()`/`expect()` with proper error propagation).
4. Re-run the fuzzer for another 30 s to confirm the crash is gone.
5. Commit the fix and the updated target.
