/// Property-based round-trip tests for invertible CLI operations.
///
/// Each test drives the built binary via assert_cmd (consistent with the rest
/// of the test suite). proptest generates many inputs and asserts invariants
/// that fixed unit tests cannot cover.
///
/// Case counts are kept modest (64) because each case spawns a process.
use assert_cmd::Command;
use proptest::prelude::*;

/// Run ubertool, passing `input` through stdin (no positional arg) so that
/// strings starting with `-` are not misinterpreted as flags, and strings
/// containing NUL cannot crash the OS arg vector. Returns stdout with the
/// trailing newline stripped.
///
/// Panics if the command exits non-zero.
fn ut_stdin(args: &[&str], input: &str) -> String {
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(args)
        .write_stdin(input.to_string())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out)
        .unwrap()
        .trim_end_matches('\n')
        .to_string()
}

/// Variant that passes `value` as a positional arg — safe only for strings
/// guaranteed not to start with `-` and not to contain NUL.
fn ut_arg(args: &[&str], value: &str) -> String {
    let mut full_args: Vec<&str> = args.to_vec();
    full_args.push(value);
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(&full_args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out)
        .unwrap()
        .trim_end_matches('\n')
        .to_string()
}

proptest! {
    // Keep cases at 64 — each case spawns a subprocess.
    #![proptest_config(ProptestConfig::with_cases(64))]

    // ── 1. base64 round-trip ──────────────────────────────────────────────
    // decode(encode(x)) == x for arbitrary printable-ASCII strings.
    // Input is piped via stdin to avoid the `-foo` flag ambiguity.
    // Newlines in the input are excluded because `to_binary`/trim_end
    // would eat a trailing one; they are valid base64 payload but the
    // extra trimming makes the comparison unreliable without --in file.
    #[test]
    fn base64_roundtrip(s in "[ -~]{0,128}") {
        // encode via stdin, decode via stdin
        let encoded = ut_stdin(&["base64", "encode"], &s);
        let decoded = ut_stdin(&["base64", "decode"], &encoded);
        prop_assert_eq!(decoded, s);
    }

    // ── 2. URL round-trip ─────────────────────────────────────────────────
    // decode(encode(x)) == x for arbitrary UTF-8 strings.
    // NUL bytes cannot appear in Rust strings (they are valid Unicode scalar
    // values U+0000 but the strategy excludes them for safety with assert_cmd
    // stdin). Newlines excluded: trailing-newline trimming would eat one.
    #[test]
    fn url_roundtrip(s in "[^\x00\n\r]{0,64}") {
        let encoded = ut_stdin(&["url", "encode"], &s);
        let decoded = ut_stdin(&["url", "decode"], &encoded);
        prop_assert_eq!(decoded, s);
    }

    // ── 3. HTML round-trip ────────────────────────────────────────────────
    // decode(encode(x)) == x for arbitrary UTF-8 strings.
    // html encode escapes <, >, &, ", ' — decode reverses all of them.
    // Newlines excluded to avoid trailing-newline trimming ambiguity.
    #[test]
    fn html_roundtrip(s in "[^\x00\n\r]{0,64}") {
        let encoded = ut_stdin(&["html", "encode"], &s);
        let decoded = ut_stdin(&["html", "decode"], &encoded);
        prop_assert_eq!(decoded, s);
    }

    // ── 4. text to-binary / from-binary round-trip (ASCII) ───────────────
    // from-binary(to-binary(x)) == x for arbitrary printable-ASCII strings.
    // to-binary emits space-separated 8-bit byte representations;
    // from-binary reverses that. Printable ASCII is " " (0x20) through "~"
    // (0x7E), which is a subset of valid UTF-8, so from-binary's UTF-8
    // check always passes. Newlines excluded to avoid trimming issues.
    #[test]
    fn text_binary_roundtrip(s in "[ -~]{0,64}") {
        let binary = ut_stdin(&["text", "to-binary"], &s);
        let recovered = ut_stdin(&["text", "from-binary"], &binary);
        prop_assert_eq!(recovered, s);
    }

    // ── 4b. text to-binary / from-binary for arbitrary UTF-8 ─────────────
    // Same invariant but with arbitrary Unicode strings. to-binary operates
    // on raw bytes (UTF-8 encoded), so the round-trip must be exact for any
    // valid UTF-8 input. NUL and newlines excluded (newline causes trimming
    // ambiguity; NUL is excluded from Rust regex strategies on this target).
    #[test]
    fn text_binary_utf8_roundtrip(s in "[^\x00\n\r]{0,32}") {
        let binary = ut_stdin(&["text", "to-binary"], &s);
        let recovered = ut_stdin(&["text", "from-binary"], &binary);
        prop_assert_eq!(recovered, s);
    }

    // ── 5. text to-unicode / from-unicode round-trip ─────────────────────
    // from-unicode(to-unicode(x)) == x for arbitrary Unicode strings.
    // to-unicode emits "U+XXXX" codepoint tokens; from-unicode reverses.
    // Surrogate codepoints (U+D800..=U+DFFF) cannot appear in Rust strings.
    // NUL and newlines excluded for the same reasons as above.
    #[test]
    fn text_unicode_roundtrip(s in "[^\x00\n\r]{0,32}") {
        let unicode = ut_stdin(&["text", "to-unicode"], &s);
        let recovered = ut_stdin(&["text", "from-unicode"], &unicode);
        prop_assert_eq!(recovered, s);
    }

    // ── 6. JSON idempotent minify ─────────────────────────────────────────
    // minify(prettify(minify(x))) == minify(x) for valid JSON documents.
    // Generating arbitrary valid JSON via proptest is unwieldy, so we use
    // a prop_oneof! over a hand-curated collection of representative
    // documents. The property (stable minification) is what matters.
    #[test]
    fn json_minify_idempotent(
        doc in prop_oneof![
            Just(r#"{"a":1}"#.to_string()),
            Just(r#"{"a":1,"b":"hello","c":true,"d":null}"#.to_string()),
            Just(r#"{"arr":[1,2,3],"nested":{"x":42}}"#.to_string()),
            Just(r#"[1,2,3,4,5]"#.to_string()),
            Just(r#"[]"#.to_string()),
            Just(r#"{}"#.to_string()),
            Just(r#"{"key":"value with spaces"}"#.to_string()),
            Just(r#"{"unicode":"ñ café"}"#.to_string()),
            Just(r#"{"numbers":[0,-1,3.14,1e10]}"#.to_string()),
            Just(r#"{"deep":{"a":{"b":{"c":"leaf"}}}}"#.to_string()),
            Just(r#"{"bool_array":[true,false,true]}"#.to_string()),
            Just(r#"{"mixed":[1,"two",null,true,{"nested":3}]}"#.to_string()),
        ]
    ) {
        let minified_first = ut_stdin(&["json", "minify"], &doc);
        let prettified = ut_stdin(&["json", "prettify"], &minified_first);
        let minified_second = ut_stdin(&["json", "minify"], &prettified);
        prop_assert_eq!(minified_second, minified_first,
            "minify(prettify(minify(doc))) != minify(doc) for doc: {}", doc);
    }

    // ── 7. integer-base round-trip ────────────────────────────────────────
    // Converting n from base 10 → base B → base 10 yields n.
    // n in 0..=1_000_000, base in 2..=36.
    // Values are always numeric/alphanumeric (no leading `-`), so passing
    // as positional args is safe here.
    #[test]
    fn integer_base_roundtrip(n in 0u64..=1_000_000u64, base in 2u32..=36u32) {
        let n_str = n.to_string();
        let base_str = base.to_string();
        // Convert n (base 10) to base B.
        let in_base_b = ut_arg(
            &["integer-base", "convert", "--from", "10", "--to", &base_str],
            &n_str,
        );
        // Convert result back to base 10.
        let back_to_10 = ut_arg(
            &["integer-base", "convert", "--from", &base_str, "--to", "10"],
            &in_base_b,
        );
        prop_assert_eq!(back_to_10, n_str,
            "round-trip failed: {} -> base {} -> \"{}\" -> base 10", n, base, in_base_b);
    }

    // ── 8. roman numeral round-trip ───────────────────────────────────────
    // to-num(from-num(n)) == n for n in 1..=3999.
    // roman from-num <n> → roman numerals, roman to-num <roman> → n.
    // Both steps produce alphanumeric output, safe as positional args.
    #[test]
    fn roman_roundtrip(n in 1u32..=3999u32) {
        let n_str = n.to_string();
        let roman = ut_arg(&["roman", "from-num"], &n_str);
        let recovered = ut_arg(&["roman", "to-num"], &roman);
        prop_assert_eq!(recovered, n_str,
            "roman round-trip failed: {} -> \"{}\" -> ?", n, roman);
    }
}
