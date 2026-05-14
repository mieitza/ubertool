//! TTY detection wrappers. Centralized so the rest of the codebase never
//! talks to `std::io::IsTerminal` directly — makes the "stdin is a pipe vs
//! TTY" decision (design-spec §6) testable and mockable.

use std::io::IsTerminal;

pub fn is_stdin_tty() -> bool {
    std::io::stdin().is_terminal()
}

pub fn is_stdout_tty() -> bool {
    std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callable() {
        // The booleans depend on how cargo runs the test (usually no TTY).
        // We just verify the functions are callable and return a bool.
        let _ = is_stdin_tty();
        let _ = is_stdout_tty();
    }
}
