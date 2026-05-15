//! Lowercase hex encoder used by every hash- and MAC-emitting command.

pub fn encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_empty() {
        assert_eq!(encode(&[]), "");
    }

    #[test]
    fn single_byte_pads_to_two_chars() {
        assert_eq!(encode(&[0x00]), "00");
        assert_eq!(encode(&[0x0f]), "0f");
        assert_eq!(encode(&[0xff]), "ff");
    }

    #[test]
    fn known_pattern() {
        assert_eq!(encode(b"\x00\x01\x02\x03"), "00010203");
    }
}
