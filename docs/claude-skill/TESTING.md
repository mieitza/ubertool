# Testing the ubertool Claude skill end-to-end

This walks through verifying that the `ubertool` agent skill (a) gets discovered by Claude, (b) bootstraps the binary via the one-liner when it's missing, and (c) drives ubertool correctly.

Run this in a **fresh Claude Code session** on a machine where `ubertool` is **not** installed (or temporarily move it aside: `mv ~/.local/bin/ubertool /tmp/`).

## 1. Install just the skill (not the binary)

```bash
mkdir -p ~/.claude/skills/ubertool
curl -fsSL https://raw.githubusercontent.com/mieitza/ubertool/main/docs/claude-skill/SKILL.md \
  -o ~/.claude/skills/ubertool/SKILL.md
```

Confirm the binary is absent:

```bash
which ubertool || echo "ubertool not installed — good, that's the test condition"
```

## 2. Start a fresh Claude Code session and run these prompts

Run them one at a time. Expected behavior is noted under each.

### Prompt A — skill discovery + bootstrap

> "SHA-256 hash the string `hello world` for me."

**Expected:** Claude should recognize this matches the ubertool skill's description (data transform / hash). Because `ubertool` isn't on PATH, the skill instructs it to run the bootstrap one-liner first:
`which ubertool || curl -fsSL .../install.sh | sh`
Then it runs `ubertool hash sha256 "hello world"` and reports the digest.

**Pass criteria:**
- Claude installs ubertool without being explicitly told to.
- The final answer is `b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9`.

### Prompt B — JSON mode preference

> "Give me a new UUID as JSON."

**Expected:** Claude runs `ubertool --json uuid new` (not the bare form), because the skill says to prefer `--json` for structured consumption.

**Pass criteria:** Output is `{"uuid":"..."}`, a valid v4 UUID.

### Prompt C — exit-code branching

> "Verify this JWT against the secret `wrong`: `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dummy` — and tell me whether the secret is wrong or the token itself is malformed."

**Expected:** Claude runs `ubertool jwt verify ... --secret wrong --json`, inspects the exit code, and distinguishes exit 3 (malformed token) from exit 5 (signature mismatch / wrong secret) per the skill's exit-code table.

**Pass criteria:** Claude correctly identifies this token as malformed (it has a fake signature segment) → exit 3, `invalid_jwt` — *not* a "wrong secret" conclusion.

### Prompt D — discovery of an unknown verb

> "Use ubertool to convert the CIDR block `192.168.1.0/26` into its subnet details."

**Expected:** Claude runs `ubertool ipv4 subnet 192.168.1.0/26 --json` (possibly after `ubertool ipv4 --help` to confirm the verb name).

**Pass criteria:** Output includes `network`, `broadcast`, `first_host`, `last_host`, `host_count: 62`.

### Prompt E — anti-pattern boundary

> "Use ubertool to fetch the current weather in Paris."

**Expected:** Per the skill's "When NOT to use ubertool" section, Claude declines — ubertool is offline, no HTTP. It should not invent a `ubertool weather` command.

**Pass criteria:** Claude explains ubertool can't do network requests and does not hallucinate a verb.

### Prompt F — schema introspection

> "What nouns does ubertool support?"

**Expected:** Claude should run `ubertool schema --json` (or `ubertool schema --json | jq '.commands | keys'`) and answer from the structured output — not from memory or by guessing. The skill says `schema --json` is the preferred agent discovery move.

**Pass criteria:**
- Claude invokes `ubertool schema --json` (not `ubertool --help` parsed as a table, not a list from training memory).
- The answer cites nouns visible in the schema output (e.g. `hash`, `vault`, `jwt`, `ipv4`, etc.).
- Claude does not fabricate nouns that aren't present.

### Prompt G — vault usage

> "I have an API key I need to store securely for later use with ubertool."

**Expected:** Claude should walk the user through the vault workflow: `vault init` (if needed), `vault set <name>`, and explain how to retrieve it later with `vault get` or via `UBERTOOL_VAULT_PASSWORD` env for headless flows.

**Pass criteria:**
- Claude suggests `ubertool vault init` and `ubertool vault set <name>`.
- Claude explains `ubertool vault get <name>` or `--secret "$(ubertool vault get <name>)"` for consumption.
- Claude mentions the `UBERTOOL_VAULT_PASSWORD` env var for non-interactive (CI/headless) flows.
- Claude does NOT suggest storing the master password in conversation context or a plaintext file.

### Prompt H — batch hashing

> "Hash these 5 strings with SHA-256: alpha, beta, gamma, delta, epsilon."

**Expected:** Claude should pipe all 5 strings through `ubertool hash sha256 --batch` in a single invocation, not run 5 separate `ubertool hash sha256` calls.

**Pass criteria:**
- Claude uses `printf` (or equivalent) to pipe all inputs and runs exactly one `ubertool hash sha256 --batch` command.
- Output is JSONL (one `{"hash":"...","input":"..."}` per line).
- Claude does not invoke `ubertool hash sha256` five separate times.

## 3. Record results

For each prompt A–H, note PASS / FAIL and any surprises:

| Prompt | Pass? | Notes |
|--------|-------|-------|
| A — bootstrap | | |
| B — json mode | | |
| C — exit codes | | |
| D — discovery | | |
| E — boundary | | |
| F — schema introspection | | |
| G — vault usage | | |
| H — batch hashing | | |

## 4. Common failure modes and fixes

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Claude never mentions ubertool | Skill `description` didn't trigger | Tighten/expand trigger keywords in the frontmatter `description`. |
| Claude tries to install but the one-liner fails | Repo private, or `~/.local/bin` not writable | Confirm repo is public; check `--bin-dir` perms. |
| Claude parses table output instead of `--json` | Skill didn't emphasize `--json` enough | The "Rules for invocation" section already says "always prefer `--json`" — check Claude actually read the skill body. |
| Claude hallucinates a verb | Surface map too vague | The skill says "Run `ubertool --help` to confirm before assuming a verb exists" — verify that line is present. |
| Exit-code confusion (3 vs 5) | Skill's exit-code table not consulted | The table is explicit; this is a model-attention issue more than a skill issue. |

## 5. Restore your environment

If you moved the real binary aside:

```bash
mv /tmp/ubertool ~/.local/bin/ 2>/dev/null || true
```
