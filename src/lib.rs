#![doc = concat!("[![crates.io](https://img.shields.io/crates/v/", env!("CARGO_PKG_NAME"), "?style=flat-square&logo=rust)](https://crates.io/crates/", env!("CARGO_PKG_NAME"), ")")]
#![doc = concat!("[![docs.rs](https://img.shields.io/docsrs/", env!("CARGO_PKG_NAME"), "?style=flat-square&logo=docs.rs)](https://docs.rs/", env!("CARGO_PKG_NAME"), ")")]
#![doc = "![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)"]
#![doc = concat!("![msrv](https://img.shields.io/badge/msrv-", env!("CARGO_PKG_RUST_VERSION"), "-blue?style=flat-square&logo=rust)")]
//! [![github](https://img.shields.io/github/stars/nik-rev/serde-cursor)](https://github.com/nik-rev/serde-cursor)
//!
//! This crate allows you to declaratively specify how to fetch the desired parts of a serde-compatible data format
//! efficiently, without loading it into memory, using a [jq](https://jqlang.org/tutorial/)-like language.
//!
//! ```toml
#![doc = concat!(env!("CARGO_PKG_NAME"), " = ", "\"", env!("CARGO_PKG_VERSION_MAJOR"), ".", env!("CARGO_PKG_VERSION_MINOR"), "\"")]
//! ```
//!
//! # Examples
//!
//! The [`Cursor!`] macro makes it extremely easy to extract nested fields from data.
//!
//! ## Get version from `Cargo.toml`
//!
//! ```toml
//! # Cargo.toml
//! [workspace.package]
//! version = "0.1"
//! ```
//!
//! Accessed with `workspace.package.version`:
//!
//! ```
//! # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from("workspace = { package = { version = '0.1' } }")) } }
//! use serde_cursor::Cursor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = fs::read_to_string("Cargo.toml")?;
//!
//! let version: String = toml::from_str::<Cursor!(workspace.package.version)>(&data)?.0;
//! assert_eq!(version, "0.1");
//! # Ok(()) }
//! ```
//!
//! **Without `serde_cursor`**:
//!
//! *Pain and suffering...*
//!
//! ```
//! # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from("workspace = { package = { version = '0' } }")) } }
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
//! let data = fs::read_to_string("Cargo.toml")?;
//!
//! let version = toml::from_str::<CargoToml>(&data)?.workspace.package.version;
//! # Ok(()) }
//! ```
//!
//! ## Get names of all dependencies from `Cargo.lock`
//!
//! ```toml
//! [[package]]
//! serde = "1.0"
//!
//! [[package]]
//! rand = "0.9"
//! ```
//!
//! The wildcard `.*` accesses every element in an array:
//!
//! ```
//! # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from("package = [{ name = 'serde' }, { name = 'rand' }]")) } }
//! use serde_cursor::Cursor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = fs::read_to_string("Cargo.lock")?;
//!
//! let packages: Vec<String> = toml::from_str::<Cursor!(package.*.name)>(&file)?.0;
//!
//! assert_eq!(packages, vec!["serde", "rand"]);
//! # Ok(()) }
//! ```
//!
//! **Without `serde_cursor`**:
//!
//! ```
//! # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from("package = [{ name = '' }]")) } }
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CargoLock {
//!     package: Vec<Package>
//! }
//!
//! #[derive(Deserialize)]
//! struct Package {
//!     name: String
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = fs::read_to_string("Cargo.lock")?;
//!
//! let packages = toml::from_str::<CargoLock>(&file)?
//!     .package
//!     .into_iter()
//!     .map(|pkg| pkg.name)
//!     .collect::<Vec<_>>();
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
//! # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from(r#"{ "commits": [{"author": ""}] }"#)) } }
//! use serde_cursor::Cursor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = fs::read_to_string("data.json")?;
//!
//! let authors: Vec<String> = serde_json::from_str::<Cursor!(commits.*.author)>(&data)?.0;
//! # Ok(()) }
//! ```
//!
//! `serde_query`:
//!
//! ```
//! # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from(r#"{ "commits": [{"author": ""}] }"#)) } }
//! use serde_query::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Data {
//!     #[query(".commits.[].author")]
//!     authors: Vec<String>,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = fs::read_to_string("data.json")?;
//! let data: Data = serde_json::from_str(&data)?;
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
//! # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from(r#"{ "count": 0, "commits": [{"author": ""}] }"#)) } }
//! use serde::Deserialize;
//! use serde_cursor::Cursor;
//!
//! #[derive(Deserialize)]
//! struct Data {
//!     #[serde(rename = "commits")]
//!     authors: Cursor!(*.author: Vec<String>),
//!     count: usize,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = fs::read_to_string("data.json")?;
//!
//! let data: Data = serde_json::from_str(&data)?;
//! # Ok(()) }
//! ```
//!
//! `serde_query`:
//!
//! ```
//! # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from(r#"{ "count": 0, "commits": [{"author": ""}] }"#)) } }
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
//! let data = fs::read_to_string("data.json")?;
//!
//! let data: Data = serde_json::from_str(&data)?;
//! # Ok(()) }
//! ```
//!
//! # `serde_with` integration
//!
//! If `feature = "serde_with"` is enabled, [`Cursor`](struct@Cursor) will implement [`serde_with::DeserializeAs`] and [`serde_with::SerializeAs`],
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
//! # Great error messages
//!
//! When deserialization fails, you get the exact path of where the failure occurred.
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
//! # How does it work?
//!
//! The [`Cursor!`] macro is a "type-level" parser. It takes your jq-like query and transforms it into a nested, recursive type that implements [`serde::Deserialize`](serde_core::Deserialize).
//!
//! Consider this query, which gets the first dependency of every dependency in `Cargo.toml`:
//!
//! ```rust
//! # /*
//! Cursor!(package.*.dependencies.0: String)
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
//! That macro is expanded into a [Cursor](struct@Cursor) type, which implements [Deserialize](serde_core::Deserialize) and [Serialize](serde_core::Serialize):
//!
//! ```rust
//! # /*
//! Cursor<
//!     String,
//!     Cons<
//!         Field<"package">,
//!         Cons<
//!             Wildcard,
//!             Cons<
//!                 Field<"dependencies">,
//!                 Cons<Index<0>, Nil>,
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
//! vec!["package", *, "dependencies", 0]
//! # */
//! ```
//!
//! Except it exists entirely in the type system.
//!
//! Each time the [`Deserialize::deserialize()`](https://docs.rs/serde/latest/serde/trait.Deserialize.html#tymethod.deserialize) function is called, the first element of the type-level list is removed,
//! and the rest of the list is passed to the [`Deserialize`](serde_core::Deserialize) trait, again.
//!
//! This happens until the list is exhausted, in which case we finally get to the type of the field - the `String` in the above example,
//! and finally call [`Deserialize::deserialize()`](https://docs.rs/serde/latest/serde/trait.Deserialize.html#tymethod.deserialize) on that, to finish things off.
#![allow(rustdoc::invalid_rust_codeblocks)]

mod de;
mod path_segment;
mod ser;

use core::fmt;
use core::marker::PhantomData;

pub use de::DeserializePath;
pub use path_segment::ConstPathSegment;
pub use path_segment::FieldName;
pub use path_segment::Index;
pub use path_segment::PathSegment;
pub use ser::SerializeCursor;
/// Access nested fields of values easily.
///
/// ```toml
/// # Cargo.toml
/// [workspace.package]
/// version = "0.1"
/// ```
///
/// To access nested fields, use dotted field syntax:
///
/// ```
/// # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from("workspace = { package = { version = '0.1' } }")) } }
/// use serde_cursor::Cursor;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let data = fs::read_to_string("Cargo.toml")?;
///
/// let version: String = toml::from_str::<Cursor!(workspace.package.version)>(&data)?.0;
/// assert_eq!(version, "0.1");
/// # Ok(()) }
/// ```
///
/// You can access elements of arrays:
///
/// ```toml
/// # Cargo.toml
/// [workspace.package]
/// version = "0.1"
/// ```
///
/// See the [crate-level](crate) documentation for more.
#[doc(inline)]
pub use serde_cursor_impl::Cursor;

/// Type returned by the [`Cursor!`] macro.
pub struct Cursor<T, P>(pub T, #[doc(hidden)] pub PhantomData<P>);

impl<T, P> From<T> for Cursor<T, P> {
    fn from(value: T) -> Self {
        Self(value, PhantomData)
    }
}

impl<T, P> std::ops::Deref for Cursor<T, P> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: fmt::Debug, P> fmt::Debug for Cursor<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <T as fmt::Display>::fmt(&self.0, f)
    }
}

impl<T: core::hash::Hash, P> core::hash::Hash for Cursor<T, P> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
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
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        <T as PartialOrd>::partial_cmp(&self.0, &other.0)
    }
}

impl<T: Ord, P> core::cmp::Ord for Cursor<T, P> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        <T as Ord>::cmp(&self.0, &other.0)
    }
}

pub mod const_str;

// This only exists to make the generated macro output
// slightly more sane

#[doc(hidden)]
pub use const_str::Char1Byte as C1;
#[doc(hidden)]
pub use const_str::Char2Byte as C2;
#[doc(hidden)]
pub use const_str::Char3Byte as C3;
#[doc(hidden)]
pub use const_str::Char4Byte as C4;
#[doc(hidden)]
pub use const_str::StrLen;

/// Represents the end of a list.
pub struct Nil;

/// Represents a single segment of a serde path.
pub struct Cons<S, P>(PhantomData<(S, P)>);

/// Represents the `*` in `Cursor!(package.*.name)`.
pub struct Wildcard;

mod sequence;
pub use sequence::Sequence;
