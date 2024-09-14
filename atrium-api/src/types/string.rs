//! Lexicon [string formats].
//!
//! [string formats]: https://atproto.com/specs/lexicon#string-formats

use chrono::DurationRound;
use ipld_core::cid;
use langtag::{LanguageTag, LanguageTagBuf};
use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::{cell::OnceCell, cmp, ops::Deref, str::FromStr};

/// Common trait implementations for Lexicon string formats that are newtype wrappers
/// around `String`.
macro_rules! string_newtype {
    ($name:ident) => {
        impl FromStr for $name {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s.into())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = Deserialize::deserialize(deserializer)?;
                Self::new(value).map_err(D::Error::custom)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }
    };
}

/// An AT Protocol identifier.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(untagged)]
pub enum AtIdentifier {
    Did(Did),
    Handle(Handle),
}

impl From<Did> for AtIdentifier {
    fn from(did: Did) -> Self {
        AtIdentifier::Did(did)
    }
}

impl From<Handle> for AtIdentifier {
    fn from(handle: Handle) -> Self {
        AtIdentifier::Handle(handle)
    }
}

impl FromStr for AtIdentifier {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(did) = s.parse() {
            Ok(AtIdentifier::Did(did))
        } else {
            s.parse().map(AtIdentifier::Handle)
        }
    }
}

impl From<AtIdentifier> for String {
    fn from(value: AtIdentifier) -> Self {
        match value {
            AtIdentifier::Did(did) => did.into(),
            AtIdentifier::Handle(handle) => handle.into(),
        }
    }
}

impl AsRef<str> for AtIdentifier {
    fn as_ref(&self) -> &str {
        match self {
            AtIdentifier::Did(did) => did.as_ref(),
            AtIdentifier::Handle(handle) => handle.as_ref(),
        }
    }
}

/// A [CID in string format].
///
/// [CID in string format]: https://atproto.com/specs/data-model#link-and-cid-formats
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cid(cid::Cid);

impl Cid {
    /// Prepares a CID for use as a Lexicon string.
    pub fn new(cid: cid::Cid) -> Self {
        Self(cid)
    }
}

impl FromStr for Cid {
    type Err = cid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl<'de> Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&value).map_err(D::Error::custom)
    }
}

impl Serialize for Cid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl AsRef<cid::Cid> for Cid {
    fn as_ref(&self) -> &cid::Cid {
        &self.0
    }
}

/// A Lexicon timestamp.
#[derive(Clone, Debug, Eq)]
pub struct Datetime {
    /// Serialized form. Preserved during parsing to ensure round-trip re-serialization.
    serialized: String,
    /// Parsed form.
    dt: chrono::DateTime<chrono::FixedOffset>,
}

impl PartialEq for Datetime {
    fn eq(&self, other: &Self) -> bool {
        self.dt == other.dt
    }
}

impl Ord for Datetime {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.dt.cmp(&other.dt)
    }
}

impl PartialOrd for Datetime {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Datetime {
    /// Returns a `Datetime` which corresponds to the current date and time in UTC.
    ///
    /// The timestamp uses microsecond precision.
    pub fn now() -> Self {
        Self::new(chrono::Utc::now().fixed_offset())
    }

    /// Constructs a new Lexicon timestamp.
    ///
    /// The timestamp is rounded to microsecond precision.
    pub fn new(dt: chrono::DateTime<chrono::FixedOffset>) -> Self {
        let dt = dt
            .duration_round(chrono::Duration::microseconds(1))
            .expect("delta does not exceed limits");
        // This serialization format is compatible with ISO 8601.
        let serialized = dt.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
        Self { serialized, dt }
    }

    /// Extracts a string slice containing the entire `Datetime`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.serialized.as_str()
    }
}

impl FromStr for Datetime {
    type Err = chrono::ParseError;

    #[allow(
        clippy::borrow_interior_mutable_const,
        clippy::declare_interior_mutable_const
    )]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // The `chrono` crate only supports RFC 3339 parsing, but Lexicon restricts
        // datetimes to the subset that is also valid under ISO 8601. Apply a regex that
        // validates enough of the relevant ISO 8601 format that the RFC 3339 parser can
        // do the rest.
        const RE_ISO_8601: OnceCell<Regex> = OnceCell::new();
        if RE_ISO_8601
            .get_or_init(|| Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(\.[0-9]+)?(Z|(\+[0-9]{2}|\-[0-9][1-9]):[0-9]{2})$").unwrap())
            .is_match(s)
        {
            let dt = chrono::DateTime::parse_from_rfc3339(s)?;
            Ok(Self {
                serialized: s.into(),
                dt,
            })
        } else {
            // Simulate an invalid `ParseError`.
            Err(chrono::DateTime::parse_from_rfc3339("invalid").expect_err("invalid"))
        }
    }
}

impl<'de> Deserialize<'de> for Datetime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&value).map_err(D::Error::custom)
    }
}

impl Serialize for Datetime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.serialized)
    }
}

impl AsRef<chrono::DateTime<chrono::FixedOffset>> for Datetime {
    fn as_ref(&self) -> &chrono::DateTime<chrono::FixedOffset> {
        &self.dt
    }
}

/// A generic [DID Identifier].
///
/// [DID Identifier]: https://atproto.com/specs/did
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Hash)]
#[serde(transparent)]
pub struct Did(String);
string_newtype!(Did);

impl Did {
    #[allow(
        clippy::borrow_interior_mutable_const,
        clippy::declare_interior_mutable_const
    )]
    /// Parses a `Did` from the given string.
    pub fn new(did: String) -> Result<Self, &'static str> {
        const RE_DID: OnceCell<Regex> = OnceCell::new();

        // https://atproto.com/specs/did#at-protocol-did-identifier-syntax
        if did.len() > 2048 {
            Err("DID too long")
        } else if !RE_DID
            .get_or_init(|| Regex::new(r"^did:[a-z]+:[a-zA-Z0-9._:%-]*[a-zA-Z0-9._-]$").unwrap())
            .is_match(&did)
        {
            Err("Invalid DID")
        } else {
            Ok(Self(did))
        }
    }

    /// Returns the DID method.
    pub fn method(&self) -> &str {
        &self.0[..4 + self.0[4..].find(':').unwrap()]
    }

    /// Returns the DID as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A [Handle Identifier].
///
/// [Handle Identifier]: https://atproto.com/specs/handle
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Hash)]
#[serde(transparent)]
pub struct Handle(String);
string_newtype!(Handle);

impl Handle {
    #[allow(
        clippy::borrow_interior_mutable_const,
        clippy::declare_interior_mutable_const
    )]
    /// Parses a `Handle` from the given string.
    pub fn new(handle: String) -> Result<Self, &'static str> {
        const RE_HANDLE: OnceCell<Regex> = OnceCell::new();

        // https://atproto.com/specs/handle#handle-identifier-syntax
        if handle.len() > 253 {
            Err("Handle too long")
        } else if !RE_HANDLE
            .get_or_init(|| Regex::new(r"^([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$").unwrap())
            .is_match(&handle)
        {
            Err("Invalid handle")
        } else {
            Ok(Self(handle))
        }
    }

    /// Returns the handle as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A [Namespaced Identifier].
///
/// [Namespaced Identifier]: https://atproto.com/specs/nsid
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Hash)]
#[serde(transparent)]
pub struct Nsid(String);
string_newtype!(Nsid);

impl Nsid {
    #[allow(
        clippy::borrow_interior_mutable_const,
        clippy::declare_interior_mutable_const
    )]
    /// Parses an NSID from the given string.
    pub fn new(nsid: String) -> Result<Self, &'static str> {
        const RE_NSID: OnceCell<Regex> = OnceCell::new();

        // https://atproto.com/specs/handle#handle-identifier-syntax
        if nsid.len() > 317 {
            Err("NSID too long")
        } else if !RE_NSID
            .get_or_init(|| Regex::new(r"^[a-zA-Z]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+(\.[a-zA-Z]([a-zA-Z]{0,61}[a-zA-Z])?)$").unwrap())
            .is_match(&nsid)
        {
            Err("Invalid NSID")
        } else {
            Ok(Self(nsid))
        }
    }

    /// Returns the domain authority part of the NSID.
    pub fn domain_authority(&self) -> &str {
        let split = self.0.rfind('.').expect("enforced by constructor");
        &self.0[..split]
    }

    /// Returns the name segment of the NSID.
    pub fn name(&self) -> &str {
        let split = self.0.rfind('.').expect("enforced by constructor");
        &self.0[split + 1..]
    }

    /// Returns the NSID as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// An [IETF Language Tag] string.
///
/// [IETF Language Tag]: https://en.wikipedia.org/wiki/IETF_language_tag
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Hash)]
#[serde(transparent)]
pub struct Language(LanguageTagBuf);

impl Language {
    /// Creates a new language tag by parsing the given string.
    pub fn new(s: String) -> Result<Self, langtag::Error> {
        LanguageTagBuf::new(s.into()).map(Self).map_err(|(e, _)| e)
    }

    /// Returns a [`LanguageTag`] referencing this tag.
    #[inline]
    pub fn as_ref(&self) -> LanguageTag {
        self.0.as_ref()
    }
}

impl FromStr for Language {
    type Err = langtag::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.into())
    }
}

impl Serialize for Language {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

/// A [Timestamp Identifier].
///
/// [Timestamp Identifier]: https://atproto.com/specs/record-key#record-key-type-tid
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Hash)]
#[serde(transparent)]
pub struct Tid(String);
string_newtype!(Tid);

impl Tid {
    #[allow(
        clippy::borrow_interior_mutable_const,
        clippy::declare_interior_mutable_const
    )]
    /// Parses a `TID` from the given string.
    pub fn new(tid: String) -> Result<Self, &'static str> {
        const RE_TID: OnceCell<Regex> = OnceCell::new();

        if tid.len() != 13 {
            Err("TID must be 13 characters")
        } else if !RE_TID
            .get_or_init(|| {
                Regex::new(r"^[234567abcdefghij][234567abcdefghijklmnopqrstuvwxyz]{12}$").unwrap()
            })
            .is_match(&tid)
        {
            Err("Invalid TID")
        } else {
            Ok(Self(tid))
        }
    }

    /// Returns the TID as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A record key (`rkey`) used to name and reference an individual record within the same
/// collection of an atproto repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Hash)]
pub struct RecordKey(String);
string_newtype!(RecordKey);

impl RecordKey {
    #[allow(
        clippy::borrow_interior_mutable_const,
        clippy::declare_interior_mutable_const
    )]
    /// Parses a `Record Key` from the given string.
    pub fn new(s: String) -> Result<Self, &'static str> {
        const RE_RKEY: OnceCell<Regex> = OnceCell::new();

        if [".", ".."].contains(&s.as_str()) {
            Err("Disallowed rkey")
        } else if !RE_RKEY
            .get_or_init(|| Regex::new(r"^[a-zA-Z0-9.\-_:~]{1,512}$").unwrap())
            .is_match(&s)
        {
            Err("Invalid rkey")
        } else {
            Ok(Self(s))
        }
    }

    /// Returns the record key as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::*;

    #[test]
    fn valid_datetime() {
        // From https://atproto.com/specs/lexicon#datetime
        for valid in [
            // preferred
            "1985-04-12T23:20:50.123Z",
            "1985-04-12T23:20:50.123456Z",
            "1985-04-12T23:20:50.120Z",
            "1985-04-12T23:20:50.120000Z",
            // supported
            "1985-04-12T23:20:50.12345678912345Z",
            "1985-04-12T23:20:50Z",
            "1985-04-12T23:20:50.0Z",
            "1985-04-12T23:20:50.123+00:00",
            "1985-04-12T23:20:50.123-07:00",
        ] {
            let json_valid = format!("\"{}\"", valid);
            let res = from_str::<Datetime>(&json_valid);
            assert!(res.is_ok(), "valid Datetime `{}` parsed as invalid", valid);
            let dt = res.unwrap();
            assert_eq!(to_string(&dt).unwrap(), json_valid);
        }
    }

    #[test]
    fn invalid_datetime() {
        // From https://atproto.com/specs/lexicon#datetime
        for invalid in [
            "1985-04-12",
            "1985-04-12T23:20Z",
            "1985-04-12T23:20:5Z",
            "1985-04-12T23:20:50.123",
            "+001985-04-12T23:20:50.123Z",
            "23:20:50.123Z",
            "-1985-04-12T23:20:50.123Z",
            "1985-4-12T23:20:50.123Z",
            "01985-04-12T23:20:50.123Z",
            "1985-04-12T23:20:50.123+00",
            "1985-04-12T23:20:50.123+0000",
            // ISO-8601 strict capitalization
            "1985-04-12t23:20:50.123Z",
            "1985-04-12T23:20:50.123z",
            // RFC-3339, but not ISO-8601
            "1985-04-12T23:20:50.123-00:00",
            "1985-04-12 23:20:50.123Z",
            // timezone is required
            "1985-04-12T23:20:50.123",
            // syntax looks ok, but datetime is not valid
            "1985-04-12T23:99:50.123Z",
            "1985-00-12T23:20:50.123Z",
        ] {
            assert!(
                from_str::<Datetime>(&format!("\"{}\"", invalid)).is_err(),
                "invalid Datetime `{}` parsed as valid",
                invalid,
            );
        }
    }

    #[test]
    fn datetime_round_trip() {
        let dt = Datetime::now();
        let encoded = to_string(&dt).unwrap();
        assert_eq!(from_str::<Datetime>(&encoded).unwrap(), dt);
    }

    #[test]
    fn valid_did() {
        // From https://atproto.com/specs/did#examples
        for valid in [
            "did:plc:z72i7hdynmk6r22z27h6tvur",
            "did:web:blueskyweb.xyz",
            "did:method:val:two",
            "did:m:v",
            "did:method::::val",
            "did:method:-:_:.",
            "did:key:zQ3shZc2QzApp2oymGvQbzP8eKheVshBHbU4ZYjeXqwSKEn6N",
        ] {
            assert!(
                from_str::<Did>(&format!("\"{}\"", valid)).is_ok(),
                "valid DID `{}` parsed as invalid",
                valid,
            );
        }
    }

    #[test]
    fn invalid_did() {
        // From https://atproto.com/specs/did#examples
        for invalid in [
            "did:METHOD:val",
            "did:m123:val",
            "DID:method:val",
            "did:method:",
            "did:method:val/two",
            "did:method:val?two",
            "did:method:val#two",
        ] {
            assert!(
                from_str::<Did>(&format!("\"{}\"", invalid)).is_err(),
                "invalid DID `{}` parsed as valid",
                invalid,
            );
        }
    }

    #[test]
    fn did_method() {
        // From https://atproto.com/specs/did#examples
        for (method, did) in [
            ("did:plc", "did:plc:z72i7hdynmk6r22z27h6tvur"),
            ("did:web", "did:web:blueskyweb.xyz"),
            ("did:method", "did:method:val:two"),
            ("did:m", "did:m:v"),
            ("did:method", "did:method::::val"),
            ("did:method", "did:method:-:_:."),
            (
                "did:key",
                "did:key:zQ3shZc2QzApp2oymGvQbzP8eKheVshBHbU4ZYjeXqwSKEn6N",
            ),
        ] {
            assert_eq!(Did::new(did.to_string()).unwrap().method(), method);
        }
    }

    #[test]
    fn valid_handle() {
        // From https://atproto.com/specs/handle#identifier-examples
        for valid in [
            "jay.bsky.social",
            "8.cn",
            "name.t--t", // not a real TLD, but syntax ok
            "XX.LCS.MIT.EDU",
            "a.co",
            "xn--notarealidn.com",
            "xn--fiqa61au8b7zsevnm8ak20mc4a87e.xn--fiqs8s",
            "xn--ls8h.test",
            "example.t", // not a real TLD, but syntax ok
            // Valid syntax, but must always fail resolution due to other restrictions:
            "2gzyxa5ihm7nsggfxnu52rck2vv4rvmdlkiu3zzui5du4xyclen53wid.onion",
            "laptop.local",
            "blah.arpa",
        ] {
            assert!(
                from_str::<Handle>(&format!("\"{}\"", valid)).is_ok(),
                "valid handle `{}` parsed as invalid",
                valid,
            );
        }
    }

    #[test]
    fn invalid_handle() {
        // From https://atproto.com/specs/handle#identifier-examples
        for invalid in [
            "jo@hn.test",
            "💩.test",
            "john..test",
            "xn--bcher-.tld",
            "john.0",
            "cn.8",
            "www.masełkowski.pl.com",
            "org",
            "name.org.",
        ] {
            assert!(
                from_str::<Handle>(&format!("\"{}\"", invalid)).is_err(),
                "invalid handle `{}` parsed as valid",
                invalid,
            );
        }
    }

    #[test]
    fn valid_nsid() {
        // From https://atproto.com/specs/nsid#examples
        for valid in [
            "com.example.fooBar",
            "net.users.bob.ping",
            "a-0.b-1.c",
            "a.b.c",
            "cn.8.lex.stuff",
        ] {
            assert!(
                from_str::<Nsid>(&format!("\"{}\"", valid)).is_ok(),
                "valid NSID `{}` parsed as invalid",
                valid,
            );
        }
    }

    #[test]
    fn invalid_nsid() {
        // From https://atproto.com/specs/nsid#examples
        for invalid in ["com.exa💩ple.thing", "com.example"] {
            assert!(
                from_str::<Nsid>(&format!("\"{}\"", invalid)).is_err(),
                "invalid NSID `{}` parsed as valid",
                invalid,
            );
        }
    }

    #[test]
    fn nsid_parts() {
        // From https://atproto.com/specs/nsid#examples
        for (nsid, domain_authority, name) in [
            ("com.example.fooBar", "com.example", "fooBar"),
            ("net.users.bob.ping", "net.users.bob", "ping"),
            ("a-0.b-1.c", "a-0.b-1", "c"),
            ("a.b.c", "a.b", "c"),
            ("cn.8.lex.stuff", "cn.8.lex", "stuff"),
        ] {
            let nsid = Nsid::new(nsid.to_string()).unwrap();
            assert_eq!(nsid.domain_authority(), domain_authority);
            assert_eq!(nsid.name(), name);
        }
    }

    #[test]
    fn valid_language() {
        // From https://www.rfc-editor.org/rfc/rfc5646.html#appendix-A
        for valid in [
            // Simple language subtag:
            "de",         // German
            "fr",         // French
            "ja",         // Japanese
            "i-enochian", // example of a grandfathered tag
            // Language subtag plus Script subtag:
            "zh-Hant", // Chinese written using the Traditional Chinese script
            "zh-Hans", // Chinese written using the Simplified Chinese script
            "sr-Cyrl", // Serbian written using the Cyrillic script
            "sr-Latn", // Serbian written using the Latin script
            // Extended language subtags and their primary language subtag counterparts:
            "zh-cmn-Hans-CN", // Chinese, Mandarin, Simplified script, as used in China
            "cmn-Hans-CN",    // Mandarin Chinese, Simplified script, as used in China
            "zh-yue-HK",      // Chinese, Cantonese, as used in Hong Kong SAR
            "yue-HK",         // Cantonese Chinese, as used in Hong Kong SAR
            // Language-Script-Region:
            "zh-Hans-CN", // Chinese written using the Simplified script as used in mainland China
            "sr-Latn-RS", // Serbian written using the Latin script as used in Serbia
            // Language-Variant:
            "sl-rozaj",       // Resian dialect of Slovenian
            "sl-rozaj-biske", // San Giorgio dialect of Resian dialect of Slovenian
            "sl-nedis",       // Nadiza dialect of Slovenian
            // Language-Region-Variant:
            "de-CH-1901", // German as used in Switzerland using the 1901 variant orthography
            "sl-IT-nedis", // Slovenian as used in Italy, Nadiza dialect
            // Language-Script-Region-Variant:
            "hy-Latn-IT-arevela", // Eastern Armenian written in Latin script, as used in Italy
            // Language-Region:
            "de-DE",  // German for Germany
            "en-US",  // English as used in the United States
            "es-419", // Spanish appropriate for the Latin America and Caribbean region using the UN region code
            // Private use subtags:
            "de-CH-x-phonebk",
            "az-Arab-x-AZE-derbend",
            // Private use registry values:
            "x-whatever",             // private use using the singleton 'x'
            "qaa-Qaaa-QM-x-southern", // all private tags
            "de-Qaaa",                // German, with a private script
            "sr-Latn-QM",             // Serbian, Latin script, private region
            "sr-Qaaa-RS",             // Serbian, private script, for Serbia
            // Tags that use extensions (examples ONLY -- extensions MUST be defined by RFC):
            "en-US-u-islamcal",
            "zh-CN-a-myext-x-private",
            "en-a-myext-b-another",
            // Invalid tags that are well-formed:
            "ar-a-aaa-b-bbb-a-ccc", // two extensions with same single-letter prefix
        ] {
            let json_valid = format!("\"{}\"", valid);
            let res = from_str::<Language>(&json_valid);
            assert!(res.is_ok(), "valid language `{}` parsed as invalid", valid);
            let dt = res.unwrap();
            assert_eq!(to_string(&dt).unwrap(), json_valid);
        }
    }

    #[test]
    fn invalid_language() {
        // From https://www.rfc-editor.org/rfc/rfc5646.html#appendix-A
        for invalid in [
            "de-419-DE", // two region tags
            // use of a single-character subtag in primary position; note that there are a
            // few grandfathered tags that start with "i-" that are valid
            "a-DE",
        ] {
            assert!(
                from_str::<Language>(&format!("\"{}\"", invalid)).is_err(),
                "invalid language `{}` parsed as valid",
                invalid,
            );
        }
    }

    #[test]
    fn valid_tid() {
        for valid in ["3jzfcijpj2z2a", "7777777777777", "3zzzzzzzzzzzz"] {
            assert!(
                from_str::<Tid>(&format!("\"{}\"", valid)).is_ok(),
                "valid TID `{}` parsed as invalid",
                valid,
            );
        }
    }

    #[test]
    fn invalid_tid() {
        for invalid in [
            // not base32
            "3jzfcijpj2z21",
            "0000000000000",
            // too long/short
            "3jzfcijpj2z2aa",
            "3jzfcijpj2z2",
            // old dashes syntax not actually supported (TTTT-TTT-TTTT-CC)
            "3jzf-cij-pj2z-2a",
            // high bit can't be high
            "zzzzzzzzzzzzz",
            "kjzfcijpj2z2a",
        ] {
            assert!(
                from_str::<Tid>(&format!("\"{}\"", invalid)).is_err(),
                "invalid TID `{}` parsed as valid",
                invalid,
            );
        }
    }

    #[test]
    fn valid_rkey() {
        // From https://atproto.com/specs/record-key#examples
        for valid in [
            "3jui7kd54zh2y",
            "self",
            "literal:self",
            "example.com",
            "~1.2-3_",
            "dHJ1ZQ",
            "pre:fix",
            "_",
        ] {
            assert!(
                from_str::<RecordKey>(&format!("\"{}\"", valid)).is_ok(),
                "valid rkey `{}` parsed as invalid",
                valid,
            );
        }
    }

    #[test]
    fn invalid_rkey() {
        // From https://atproto.com/specs/record-key#examples
        for invalid in [
            "alpha/beta",
            ".",
            "..",
            "#extra",
            "@handle",
            "any space",
            "any+space",
            "number[3]",
            "number(3)",
            "\"quote\"",
            "dHJ1ZQ==",
        ] {
            assert!(
                from_str::<RecordKey>(&format!("\"{}\"", invalid)).is_err(),
                "invalid rkey `{}` parsed as valid",
                invalid,
            );
        }
    }
}
