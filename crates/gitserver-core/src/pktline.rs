//! pkt-line encoding as defined by Git protocol.
//! Format: 4-hex-digit length prefix (includes the 4 bytes) + content.
//! Flush packet: "0000"

/// Encode a single pkt-line.
pub fn encode(data: &[u8]) -> Vec<u8> {
    let len = data.len() + 4;
    let mut buf = format!("{len:04x}").into_bytes();
    buf.extend_from_slice(data);
    buf
}

/// Encode a flush packet.
pub fn flush() -> &'static [u8] {
    b"0000"
}

/// Encode a comment line (e.g. "# service=git-upload-pack\n").
pub fn encode_comment(comment: &str) -> Vec<u8> {
    encode(format!("# {comment}\n").as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_simple_line() {
        let encoded = encode(b"hello\n");
        assert_eq!(&encoded, b"000ahello\n");
    }

    #[test]
    fn encode_empty_content() {
        let encoded = encode(b"");
        assert_eq!(&encoded, b"0004");
    }

    #[test]
    fn encode_comment_line() {
        let encoded = encode_comment("service=git-upload-pack");
        assert_eq!(&encoded, b"001e# service=git-upload-pack\n");
    }

    #[test]
    fn flush_packet() {
        assert_eq!(flush(), b"0000");
    }

    #[test]
    fn encode_ref_line() {
        let hash = "0000000000000000000000000000000000000000";
        let line = format!("{hash} refs/heads/main\n");
        let encoded = encode(line.as_bytes());
        // 4 (prefix) + 40 (hash) + 1 (space) + 15 (refs/heads/main) + 1 (\n) = 61 = 0x3d
        assert_eq!(&encoded[..4], b"003d");
    }
}
