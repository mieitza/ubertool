//! Process exit codes for ubertool. The contract is documented in the design
//! spec §8 and surfaced via `ubertool --help`.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Ok = 0,
    Generic = 1,
    Usage = 2,
    Invalid = 3,
    Io = 4,
    Crypto = 5,
    Unsupported = 6,
}

impl ExitCode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_match_design_spec() {
        assert_eq!(ExitCode::Ok.as_u8(), 0);
        assert_eq!(ExitCode::Generic.as_u8(), 1);
        assert_eq!(ExitCode::Usage.as_u8(), 2);
        assert_eq!(ExitCode::Invalid.as_u8(), 3);
        assert_eq!(ExitCode::Io.as_u8(), 4);
        assert_eq!(ExitCode::Crypto.as_u8(), 5);
        assert_eq!(ExitCode::Unsupported.as_u8(), 6);
    }
}
