# ubertool

Developer-focused data-transform utilities — a single Rust binary,
agent-friendly by construction. A CLI port of [it-tools](https://it-tools.tech).

> M0 scaffolding only. Tools land milestone-by-milestone; see
> `docs/superpowers/plans/` for the rollout.

## Status

Pre-alpha. M0 ships `base64 encode|decode` end-to-end through the full
agent-friendly contract (`--json`, typed errors with documented exit codes,
TTY fail-fast, snapshot-tested `--help`).

## Quickstart

```sh
cargo build --release
./target/release/ubertool base64 encode "hello"
./target/release/ubertool base64 encode "hello" --json
echo -n hello | ./target/release/ubertool base64 encode
```

## Output modes

`ubertool` is designed for LLM coding agents first; humans second. Every
data-returning command supports:

- default — `key: value` lines on stdout
- `--json` — flat JSON on stdout, nothing else
- `--quiet` — bare values, one per line, pipe-friendly

Errors carry a typed code, a human message, an echo of the failing input
(with secrets redacted), an optional hint, and a `retriable` boolean. The
JSON envelope is stable across commands.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | success |
| 1 | general failure |
| 2 | usage error |
| 3 | input validation error |
| 4 | i/o error |
| 5 | cryptographic / integrity failure |
| 6 | feature not built in this binary |

## Dev loop

```sh
make build          # debug build
make test           # cargo test
make ci             # fmt + clippy + test + verify-spec
make gen            # validate spec + regenerate docs/cli/ and docs/llms.txt
make verify-spec    # check ubertool.ocs.yaml ↔ --help consistency
```

`ubertool.ocs.yaml` is an [OpenCLI](https://github.com/bcdxn/opencli)
spec — the source of truth for command structure, flags, and exit codes.
The spec generates documentation; `verify-spec.sh` ensures the binary's
`--help` stays consistent with the spec.

## Design

See `docs/superpowers/specs/2026-05-15-ubertool-cli-design.md` for the
full design (command surface, output contract, crate selection, milestone
rollout). The design is rooted in two skills authored for LLM coding
agents:
- `agent-cli-design` — the eight rules for agent-callable CLIs
- `opencli-spec-author` — spec-first CLI design

## License

GPL-3.0, matching the source [it-tools](https://github.com/CorentinTh/it-tools) project.
