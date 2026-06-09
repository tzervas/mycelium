//! Content addresses (RFC-0001 §4.6): `<algo>:<digest>`.

/// A content address, e.g. `blake3:Hh3kQ_x-1A`. The concrete hash algorithm is fixed in M-103;
/// this type fixes only the shape (`<algo>:<digest>`, matching the schema pattern).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash(String);

impl ContentHash {
    /// Parse a content address, validating its shape: `algo` is `[a-z0-9]+`, `digest` is
    /// `[A-Za-z0-9_-]+`, separated by a single `:`. Returns `None` if malformed.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let (algo, digest) = s.split_once(':')?;
        if algo.is_empty() || digest.is_empty() {
            return None;
        }
        if !algo
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        {
            return None;
        }
        if !digest
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return None;
        }
        Some(ContentHash(s.to_owned()))
    }

    /// The address as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::ContentHash;

    #[test]
    fn parses_well_shaped() {
        assert!(ContentHash::parse("blake3:Hh3kQ_x-1A").is_some());
    }

    #[test]
    fn rejects_malformed() {
        assert!(ContentHash::parse("no-colon").is_none());
        assert!(ContentHash::parse("blake3:").is_none());
        assert!(ContentHash::parse(":digest").is_none());
        assert!(ContentHash::parse("UPPER:abc").is_none());
        assert!(ContentHash::parse("blake3:has space").is_none());
    }
}
