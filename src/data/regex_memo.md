# Regex cheat sheet

## Anchors
```
^         start of line/string
$         end of line/string
\b        word boundary
\B        non-word boundary
\A        start of string (Rust)
\z        end of string (Rust)
```

## Character classes
```
.         any char (except newline by default)
\d \D     digit / non-digit
\w \W     word char / non-word
\s \S     whitespace / non-whitespace
[abc]     any of a, b, c
[^abc]    none of a, b, c
[a-z]     range
```

## Quantifiers
```
?         0 or 1
*         0 or more (greedy)
+         1 or more (greedy)
*?  +? ??  lazy variants
{n}       exactly n
{n,}      n or more
{n,m}     n to m
```

## Groups
```
(abc)             capturing group
(?:abc)           non-capturing
(?P<name>abc)     named (Rust syntax)
\1 \2 ...         backrefs (NOT supported by Rust's regex crate)
```

## Lookaround (NOT supported by Rust's regex crate)
```
(?=...)   positive lookahead
(?!...)   negative lookahead
(?<=...)  positive lookbehind
(?<!...)  negative lookbehind
```

For lookaround in Rust, use the `fancy-regex` crate.

## Common patterns
```
^\d+$                            integer
^-?\d+(\.\d+)?$                  number with optional sign/decimal
^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$  email-ish
^https?://                        http/s URL prefix
\d{4}-\d{2}-\d{2}                ISO date YYYY-MM-DD
\d{2}:\d{2}(:\d{2})?             time HH:MM[:SS]
^[0-9a-fA-F]{40}$                git SHA-1
^[0-9a-fA-F]{8}(-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}$  UUID
```

## Flags (inline)
```
(?i)abc       case-insensitive
(?x)          extended (whitespace + comments)
(?m)          multiline (^/$ match line ends)
(?s)          dot matches newline
(?U)          swap greedy/lazy (Rust)
```

## Tools
- `ubertool regex test --pattern '<p>' --text '<t>'`  test a pattern
- `ubertool regex generate '<pattern>'`               generate a matching string
