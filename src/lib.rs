#![doc = concat!("[![crates.io](https://img.shields.io/crates/v/", env!("CARGO_PKG_NAME"), "?style=flat-square&logo=rust)](https://crates.io/crates/", env!("CARGO_PKG_NAME"), ")")]
#![doc = concat!("[![docs.rs](https://img.shields.io/docsrs/", env!("CARGO_PKG_NAME"), "?style=flat-square&logo=docs.rs)](https://docs.rs/", env!("CARGO_PKG_NAME"), ")")]
#![doc = "![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)"]
#![doc = concat!("![msrv](https://img.shields.io/badge/msrv-", env!("CARGO_PKG_RUST_VERSION"), "-blue?style=flat-square&logo=rust)")]
//! [![github](https://img.shields.io/github/stars/nik-rev/serde-cursor)](https://github.com/nik-rev/serde-cursor)
//!
//! This crate allows you to declaratively specify how to fetch the desired parts of a serde-compatible data format (such as JSON)
//! efficiently, without loading it all into memory, using a jq-like language.
//!
//! ```toml
#![doc = concat!(env!("CARGO_PKG_NAME"), " = ", "\"", env!("CARGO_PKG_VERSION_MAJOR"), ".", env!("CARGO_PKG_VERSION_MINOR"), "\"")]
//! ```
//!
//! # Examples
//!
//! The `Cursor!` macro makes it extremely easy to extract nested fields from data.
//!
//! ## Get version from `Cargo.toml`
//!
//! ```
//! use serde_cursor::Cursor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = r#"
//!     [workspace.package]
//!     version = "0.1"
//! "#;
//!
//! let version: String = toml::from_str::<Cursor!(workspace.package.version)>(data)?.0;
//! assert_eq!(version, "0.1");
//! # Ok(()) }
//! ```
//!
//! `Cursor!(workspace.package.version)` is the magic juice - this type-macro expands to a type that implements [`serde::Deserialize`](serde_core::Deserialize).
//!
//! **Without `serde_cursor`**:
//!
//! *Pain and suffering...*
//!
//! ```
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CargoToml {
//!     workspace: Workspace
//! }
//!
//! #[derive(Deserialize)]
//! struct Workspace {
//!     package: Package
//! }
//!
//! #[derive(Deserialize)]
//! struct Package {
//!     version: String
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = r#"
//!     [workspace.package]
//!     version = "0.1"
//! "#;
//!
//! let version = toml::from_str::<CargoToml>(data)?.workspace.package.version;
//! # Ok(()) }
//! ```
//!
//! ## Get names of all dependencies from `Cargo.lock`
//!
//! The index-all `[]` accesses every element in an array:
//!
//! ```
//! use serde_cursor::Cursor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = r#"
//!     [[package]]
//!     name = "serde"
//!
//!     [[package]]
//!     name = "rand"
//! "#;
//!
//! let packages: Vec<String> = toml::from_str::<Cursor!(package[].name)>(file)?.0;
//!
//! assert_eq!(packages, vec!["serde", "rand"]);
//! # Ok(()) }
//! ```
//!
//! # Syntax
//!
//! Specify the type `Vec<String>` after the path `package[].name`:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let file = r#"
//! #     [[package]]
//! #     name = "serde"
//! #
//! #     [[package]]
//! #     name = "rand"
//! # "#;
//! # use serde_cursor::Cursor;
//! let packages = toml::from_str::<Cursor!(package[].name: Vec<String>)>(file)?.0;
//! # Ok(()) }
//! ```
//!
//! The type can be omitted, in which case it will be inferred:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let file = r#"
//! #     [[package]]
//! #     name = "serde"
//! #
//! #     [[package]]
//! #     name = "rand"
//! # "#;
//! # use serde_cursor::Cursor;
//! let packages: Vec<String> = toml::from_str::<Cursor!(package[].name)>(file)?.0;
//! # Ok(()) }
//! ```
//!
//! Fields that consist of identifiers and `-`s can be used without quotes:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let file = r#"
//! #     [dev-dependencies.serde]
//! #     version = "1.0"
//! # "#;
//! # use serde_cursor::Cursor;
//! # let version: String = toml::from_str::<
//! Cursor!(dev-dependencies.serde.version)
//! # >(file)?.0;
//! # Ok(()) }
//! ```
//!
//! Fields that contain spaces or other special characters must be quoted:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let file = r#"
//! #     [ferris."🦀::<>"]
//! #     "\"" = true
//! # "#;
//! # use serde_cursor::Cursor;
//! # let ferris: bool = toml::from_str::<
//! Cursor!(ferris."🦀::<>".r#"""#)
//! # >(file)?.0;
//! # Ok(()) }
//! ```
//!
//! You can access specific elements of an array:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let file = r#"
//! #     [[package]]
//! #     name = "serde"
//! # "#;
//! # use serde_cursor::Cursor;
//! # let ferris: String = toml::from_str::<
//! Cursor!(package[0].name)
//! # >(file)?.0;
//! # Ok(()) }
//! ```
//!
//! # `serde_cursor` + `monostate` = 🧡💛💚💙💜
//!
//! The [`monostate`](https://github.com/dtolnay/monostate) crate provides the `MustBe!` macro, which returns a type that implements
//! [`serde::Deserialize`](serde_core::Deserialize), and can only ever deserialize from one specific value.
//!
//! Together, these 2 crates provide an almost jq-like experience of data processing in Rust:
//!
//! ```
//! # /*
//! get!(reason: MustBe!("compiler-message"))?;
//! get!(message.message: MustBe!("trace_macro"))?;
//!
//! Ok(Expansion {
//!     messages: get!(message.children[].message)?,
//!     byte_start: get!(message.spans[0].byte_start)?,
//!     byte_end: get!(message.spans[0].byte_end)?,
//! })
//! # */
//! ```
//!
//! The jq version of the above processing looks like this:
//!
//! ```jq
//! select(.reason == "compiler-message")
//! | select(.message.message == "trace_macro")
//! | {
//!     messages: [.message.children[].message],
//!     byte_start: .message.spans[0].byte_start,
//!     byte_end: .message.spans[0].byte_end
//! }
//! ```
//!
//! Considering we're comparing a strongly typed, low-level programming language with a duck-typed
//! DSL specifically designed for extracting data from JSON, I'd say the result is Not Bad™!
//!
//! The full code for the above example looks like this:
//!
//! ```
//! use monostate::MustBe;
//! use serde_cursor::Cursor;
//!
//! struct Expansion {
//!     messages: Vec<String>,
//!     byte_start: u32,
//!     byte_end: u32,
//! }
//!
//! impl Expansion {
//!     fn parse(value: &[u8]) -> serde_json::Result<Self> {
//!         macro_rules! get {
//!             ($($cursor:tt)*) => {
//!                 serde_json::from_slice::<Cursor!($($cursor)*)>(value).map(|it| it.0)
//!             };
//!         }
//!
//!         get!(reason: MustBe!("compiler-message"))?;
//!         get!(message.message: MustBe!("trace_macro"))?;
//!
//!         Ok(Expansion {
//!             messages: get!(message.children[].message)?,
//!             byte_start: get!(message.spans[0].byte_start)?,
//!             byte_end: get!(message.spans[0].byte_end)?,
//!         })
//!     }
//! }
//! ```
//!
//! <details>
//!
//! <summary>
//!
//! For reference, the same logic without `serde_cursor` or `monostate`
//!
//! </summary>
//!
//! ```
//! use serde::Deserialize;
//!
//! struct Expansion {
//!     messages: Vec<String>,
//!     byte_start: u32,
//!     byte_end: u32,
//! }
//!
//! impl Expansion {
//!     fn from_slice(value: &[u8]) -> serde_json::Result<Self> {
//!         #[derive(Deserialize)]
//!         struct RawDiagnostic {
//!             reason: String,
//!             message: DiagnosticMessage,
//!         }
//!
//!         #[derive(Deserialize)]
//!         struct DiagnosticMessage {
//!             message: String,
//!             children: Vec<DiagnosticChild>,
//!             spans: Vec<DiagnosticSpan>,
//!         }
//!
//!         #[derive(Deserialize)]
//!         struct DiagnosticChild {
//!             message: String,
//!         }
//!
//!         #[derive(Deserialize)]
//!         struct DiagnosticSpan {
//!             byte_start: u32,
//!             byte_end: u32,
//!         }
//!
//!         let raw: RawDiagnostic = serde_json::from_slice(value)?;
//!
//!         if raw.reason != "compiler-message" || raw.message.message != "trace_macro" {
//!             return Err(serde::de::Error::custom("..."));
//!         }
//!
//!         let primary_span = raw.message.spans.get(0)
//!             .ok_or_else(|| serde::de::Error::custom("..."))?;
//!
//!         Ok(Expansion {
//!             messages: raw.message.children.into_iter().map(|c| c.message).collect(),
//!             byte_start: primary_span.byte_start,
//!             byte_end: primary_span.byte_end,
//!         })
//!     }
//! }
//! ```
//!
//! </details>
//!
//! # Ranges
//!
//! Ranges are like `[]` but for only for elements with an index that falls in the range:
//!
//! ```
//! # /*
//! Cursor!(package[4..]);
//! Cursor!(package[..8]);
//! Cursor!(package[4..8]);
//! Cursor!(package[4..=8]);
//! # */
//! ```
//!
//! # Interpolations
//!
//! It's not uncommon for multiple queries to get quite repetitive:
//!
//! ```
//! # use serde_json::from_str;
//! # use serde_cursor::Cursor;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let france = "france = { properties = { timeseries = [{ data = { instant = { details = { air_pressure_at_sea_level = 1.0, relative_humidity = 2.0, air_temperature = 3.0 } } } }] } }";
//! # let japan = "japan = { properties = { timeseries = [{ data = { instant = { details = { air_pressure_at_sea_level = 4.0, relative_humidity = 5.0, air_temperature = 6.0 } } } }] } }";
//! let pressure: Vec<f64> = toml::from_str::<Cursor!(france.properties.timeseries[].data.instant.details.air_pressure_at_sea_level)>(france)?.0;
//! let humidity: Vec<f64> = toml::from_str::<Cursor!(japan.properties.timeseries[].data.instant.details.relative_humidity)>(japan)?.0;
//! let temperature: Vec<f64> = toml::from_str::<Cursor!(japan.properties.timeseries[].data.instant.details.air_temperature)>(japan)?.0;
//! # Ok(()) }
//! ```
//!
//! `serde_cursor` supports **interpolations**. You can factor out a common path into a type `Details`, and then interpolate it with `$Details` in the path inside `Cursor!`:
//!
//! ```
//! # use serde_json::from_str;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let france = "france = { properties = { timeseries = [{ data = { instant = { details = { air_pressure_at_sea_level = 1.0, relative_humidity = 2.0, air_temperature = 3.0 } } } }] } }";
//! # let japan = "japan = { properties = { timeseries = [{ data = { instant = { details = { air_pressure_at_sea_level = 4.0, relative_humidity = 5.0, air_temperature = 6.0 } } } }] } }";
//! # use serde_cursor::Cursor;
//! type Details<RestOfPath> = serde_cursor::Path!(properties.timeseries[].data.instant.details + RestOfPath);
//!
//! let pressure: Vec<f64> = toml::from_str::<Cursor!(france.$Details.air_pressure_at_sea_level)>(france)?.0;
//! let humidity: Vec<f64> = toml::from_str::<Cursor!(japan.$Details.relative_humidity)>(japan)?.0;
//! let temperature: Vec<f64> = toml::from_str::<Cursor!(japan.$Details.air_temperature)>(japan)?.0;
//! # Ok(()) }
//! ```
//!
//! # `serde_cursor` vs [`serde_query`](https://github.com/pandaman64/serde-query)
//!
//! `serde_query` also implements jq-like queries, but more verbosely.
//!
//! ## Single query
//!
//! `serde_cursor`:
//!
//! ```
//! use serde_cursor::Cursor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = r#"{ "commits": [{"author": "Ferris"}] }"#;
//!
//! let authors: Vec<String> = serde_json::from_str::<Cursor!(commits[].author)>(data)?.0;
//! # Ok(()) }
//! ```
//!
//! `serde_query`:
//!
//! ```
//! use serde_query::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Data {
//!     #[query(".commits.[].author")]
//!     authors: Vec<String>,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = r#"{ "commits": [{"author": "Ferris"}] }"#;
//! let data: Data = serde_json::from_str(data)?;
//!
//! let authors = data.authors;
//! # Ok(()) }
//! ```
//!
//! ## Storing queries in a `struct`
//!
//! `serde_cursor`:
//!
//! ```
//! use serde::Deserialize;
//! use serde_cursor::Cursor;
//!
//! #[derive(Deserialize)]
//! struct Data {
//!     #[serde(rename = "commits")]
//!     authors: Cursor!([].author: Vec<String>),
//!     count: usize,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = r#"{ "count": 1, "commits": [{"author": "Ferris"}] }"#;
//!
//! let data: Data = serde_json::from_str(data)?;
//! # Ok(()) }
//! ```
//!
//! `serde_query`:
//!
//! ```
//! use serde_query::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Data {
//!     #[query(".commits.[].author")]
//!     authors: Vec<String>,
//!     #[query(".count")]
//!     count: usize,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = r#"{ "count": 1, "commits": [{"author": "Ferris"}] }"#;
//!
//! let data: Data = serde_json::from_str(data)?;
//! # Ok(()) }
//! ```
//!
//! # Great error messages
//!
//! When deserialization fails, you get the exact path of where the failure occurred:
//!
//! ```
//! use serde_cursor::Cursor;
//!
//! let data = serde_json::json!({ "author": { "id": "not-a-number" } });
//! let result = serde_json::from_value::<Cursor!(author.id: i32)>(data);
//! let err = result.unwrap_err().to_string();
//! assert_eq!(err, r#".author.id: invalid type: string "not-a-number", expected i32"#);
//! ```
//!
//! # `serde_with` integration
//!
//! If `feature = "serde_with"` is enabled, the type returned by `Cursor!` will implement [`serde_with::DeserializeAs`](https://docs.rs/serde_with/latest/serde_with/trait.DeserializeAs.html) and [`serde_with::SerializeAs`](https://docs.rs/serde_with/latest/serde_with/trait.SerializeAs.html),
//! meaning you can use it with the `#[serde_as]` attribute:
//!
//! ```
//! # use serde_with::serde_as;
//! use serde::{Serialize, Deserialize};
//! use serde_cursor::Cursor;
//!
//! #[serde_as]
//! #[derive(Serialize, Deserialize)]
//! struct CargoToml {
//!     #[serde(rename = "workspace")]
//!     #[serde_as(as = "Cursor!(package.version)")]
//!     version: String,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let toml: CargoToml = toml::from_str("workspace = { package = { version = '0.1.0' } }")?;
//! assert_eq!(toml.version, "0.1.0");
//! assert_eq!(serde_json::to_string(&toml)?, r#"{"workspace":{"package":{"version":"0.1.0"}}}"#);
//! # Ok(()) }
//! ```
//!
//! # How does it work?
//!
//! The `Cursor!` macro expands to a recursive type that implements [`serde::Deserialize`](serde_core::Deserialize).
//! Information on how to access the nested fields is stored entirely inside the type system.
//!
//! Consider this query, which gets the first dependency of every dependency in `Cargo.toml`:
//!
//! ```rust
//! # /*
//! Cursor!(package[].dependencies[0]: String)
//! # */
//! ```
//!
//! For this `Cargo.lock`, it would extract `["libc", "find-msvc-tools"]`:
//!
//! ```toml
//! [[package]]
//! name = "android_system_properties"
//! dependencies = ["libc"]
//!
//! [[package]]
//! name = "cc"
//! dependencies = ["find-msvc-tools", "shlex"]
//! ```
//!
//! That macro is expanded into a `Cursor` type, which implements [`serde::Deserialize`](serde_core::Deserialize) and [`serde::Serialize`](serde_core::Serialize):
//!
//! ```rust
//! # /*
//! Cursor<
//!     String, // : String
//!     Path<
//!         Field<"package">, // .package
//!         Path<
//!             IndexAll, // []
//!             Path<
//!                 Field<"dependencies">, // .dependencies
//!                 Path<
//!                     Index<0>, // [0]
//!                     PathEnd
//!                 >,
//!             >,
//!         >,
//!     >,
//! >
//! # */
//! ```
//!
//! The above is essentially an equivalent to:
//!
//! ```rust
//! # /*
//! vec![
//!     Segment::Field("package"), // .package
//!     Segment::IndexAll, // []
//!     Segment::Field("dependencies"), // .dependencies
//!     Segment::Index(0) // [0]
//! ]
//! # */
//! ```
//!
//! Except it exists entirely in the type system.
//!
//! Each time the [`serde::Deserialize::deserialize()`](https://docs.rs/serde/latest/serde/trait.Deserialize.html#tymethod.deserialize) function is called,
//! the first segment of the path (`.package`) is processed, and the rest of the path (`[].dependencies[0]`) is passed to the
//! [`serde::Deserialize`](serde_core::Deserialize) trait, again, and again - until the path is empty.
//!
//! Once the path is empty, we finally get to the type of the field - the `String` in the above example,
//! and finally call [`serde::Deserialize::deserialize()`](https://docs.rs/serde/latest/serde/trait.Deserialize.html#tymethod.deserialize) on that, to finish things off -
//! this `String` is then bubbled up the stack and returned from `<Cursor<String, _> as serde::Deserialize>::deserialize`.
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc, feature(doc_cfg))]
#![allow(rustdoc::invalid_rust_codeblocks)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub(crate) mod de;
mod path_segment;
pub(crate) mod ser;

mod index;

#[doc(hidden, inline)]
pub use cursor::Cursor;
#[doc(hidden, inline)]
pub use de::DeserializePath;
#[doc(hidden, inline)]
pub use index::range::Range;
#[doc(hidden, inline)]
pub use index::range_from::RangeFrom;
#[doc(hidden, inline)]
pub use index::range_full::RangeFull;
#[doc(hidden, inline)]
pub use index::range_inclusive::RangeInclusive;
#[doc(hidden, inline)]
pub use index::range_to::RangeTo;
#[doc(hidden, inline)]
pub use index::range_to_inclusive::RangeToInclusive;
#[doc(hidden, inline)]
pub use path_segment::ConstPathSegment;
#[doc(hidden, inline)]
pub use path_segment::Field;
#[doc(hidden, inline)]
pub use path_segment::Index;
#[doc(hidden, inline)]
pub use path_segment::PathSegment;
#[doc(hidden, inline)]
pub use ser::SerializePath;
#[doc(inline)]
pub use serde_cursor_impl::Cursor;
#[doc(inline)]
pub use serde_cursor_impl::Path;

mod cursor {
    use core::fmt;
    use core::marker::PhantomData;

    /// Type returned by the [`Cursor!`](crate::Cursor!) macro.
    #[doc(hidden)]
    pub struct Cursor<T, P>(pub T, #[doc(hidden)] pub PhantomData<P>);

    impl<T, P> From<T> for Cursor<T, P> {
        fn from(value: T) -> Self {
            Self(value, PhantomData)
        }
    }

    impl<T, P> core::ops::Deref for Cursor<T, P> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: fmt::Debug, P> fmt::Debug for Cursor<T, P> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_tuple("Cursor").field(&self.0).finish()
        }
    }

    impl<T: Clone, P> Clone for Cursor<T, P> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1)
        }
    }

    impl<T: Copy, P> Copy for Cursor<T, P> {}

    impl<T: Default, P> Default for Cursor<T, P> {
        fn default() -> Self {
            Self(Default::default(), PhantomData)
        }
    }

    impl<T: fmt::Display, P> fmt::Display for Cursor<T, P> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            <T as fmt::Display>::fmt(&self.0, f)
        }
    }

    impl<T: core::hash::Hash, P> core::hash::Hash for Cursor<T, P> {
        fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
            self.0.hash(state);
            self.1.hash(state);
        }
    }

    impl<T: Eq, P> Eq for Cursor<T, P> {}

    impl<T: PartialEq, P> PartialEq for Cursor<T, P> {
        fn eq(&self, other: &Self) -> bool {
            <T as PartialEq>::eq(&self.0, &other.0)
        }
    }

    impl<T: PartialOrd, P> PartialOrd for Cursor<T, P> {
        fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
            <T as PartialOrd>::partial_cmp(&self.0, &other.0)
        }
    }

    impl<T: Ord, P> core::cmp::Ord for Cursor<T, P> {
        fn cmp(&self, other: &Self) -> core::cmp::Ordering {
            <T as Ord>::cmp(&self.0, &other.0)
        }
    }
}

/// Available if you need to implement a trait for the type returned by `Cursor!`,
/// or implement the `Sequence` trait to have the index-all `[]` syntax work with
/// more collections.
///
/// This module only shows up in the generated documentation to group items that are
/// implementation details together, but it doesn't actually exist.
///
/// All of these items are exported from the crate root, but hidden.
#[cfg(doc)]
#[doc(cfg(doc))]
pub mod implementation_details {
    #[doc(inline)]
    pub use crate::ConstPathSegment;
    #[doc(inline)]
    pub use crate::DeserializePath;
    #[doc(inline)]
    pub use crate::Field;
    #[doc(inline)]
    pub use crate::Index;
    #[doc(inline)]
    pub use crate::PathEnd;
    #[doc(inline)]
    pub use crate::Range;
    #[doc(inline)]
    pub use crate::RangeFull;
    #[doc(inline)]
    pub use crate::RangeInclusive;
    #[doc(inline)]
    pub use crate::RangeTo;
    #[doc(inline)]
    pub use crate::RangeToInclusive;
    #[doc(inline)]
    pub use crate::Sequence;
    #[doc(inline)]
    pub use crate::SerializePath;
    #[doc(inline)]
    pub use crate::const_str;
    #[doc(inline)]
    pub use crate::cursor::Cursor;
    #[doc(inline)]
    pub use crate::path::Path;
}

#[doc(hidden)]
pub mod const_str;

// This only exists to make the generated macro output
// slightly more sane

#[doc(hidden, inline)]
pub use const_str::Char1Byte as C1;
#[doc(hidden, inline)]
pub use const_str::Char2Byte as C2;
#[doc(hidden, inline)]
pub use const_str::Char3Byte as C3;
#[doc(hidden, inline)]
pub use const_str::Char4Byte as C4;
#[doc(hidden, inline)]
pub use const_str::StrLen;

mod path {
    use core::marker::PhantomData;

    /// Represents the end of the cursor path.
    #[doc(hidden)]
    pub struct PathEnd;

    /// Represents a single segment of a cursor path. This type is returned by the [`Path!`](crate::Path!) macro.
    #[doc(hidden)]
    pub struct Path<S, P>(PhantomData<(S, P)>);
}

#[doc(hidden, inline)]
pub use path::Path;
#[doc(hidden, inline)]
pub use path::PathEnd;

mod sequence;
#[doc(hidden, inline)]
pub use sequence::Sequence;
