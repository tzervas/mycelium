//! Content addresses (RFC-0001 §4.6): `<algo>:<digest>`.

use serde::{Deserialize, Serialize};

/// A content address, e.g. `blake3:Hh3kQ_x-1A`. The kernel hash is **BLAKE3** (fixed in M-103),
/// rendered as `blake3:<64-hex>`; this type fixes the shape (`<algo>:<digest>`, matching the schema
/// pattern) and stays algorithm-agnostic so a future migration is a value change, not a type change.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    /// Build a content address from an algorithm tag and digest, validating the shape (`algo` is
    /// `[a-z0-9]+`, `digest` is `[A-Za-z0-9_-]+`). Returns `None` if either part is malformed. This
    /// is the constructor the content-addressing pass (M-103) uses after computing a digest.
    #[must_use]
    pub fn from_parts(algo: &str, digest: &str) -> Option<Self> {
        Self::parse(&format!("{algo}:{digest}"))
    }

    /// The algorithm tag (the part before `:`), e.g. `blake3`.
    #[must_use]
    pub fn algo(&self) -> &str {
        self.0.split_once(':').map_or("", |(a, _)| a)
    }

    /// The digest (the part after `:`).
    #[must_use]
    pub fn digest(&self) -> &str {
        self.0.split_once(':').map_or("", |(_, d)| d)
    }

    /// The address as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for ContentHash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Validate the shape on the way in — a malformed address is an error, never silent.
        let s = String::deserialize(deserializer)?;
        ContentHash::parse(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("malformed content address: {s:?}")))
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

    #[test]
    fn from_parts_splits_back_out() {
        let h = ContentHash::from_parts("blake3", "Hh3kQ_x-1A").expect("valid");
        assert_eq!(h.algo(), "blake3");
        assert_eq!(h.digest(), "Hh3kQ_x-1A");
        assert_eq!(h.as_str(), "blake3:Hh3kQ_x-1A");
        assert!(ContentHash::from_parts("blake3", "has space").is_none());
        assert!(ContentHash::from_parts("UPPER", "abc").is_none());
    }
}
