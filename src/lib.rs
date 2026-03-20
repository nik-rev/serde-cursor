#![doc = concat!("[![crates.io](https://img.shields.io/crates/v/", env!("CARGO_PKG_NAME"), "?style=flat-square&logo=rust)](https://crates.io/crates/", env!("CARGO_PKG_NAME"), ")")]
#![doc = concat!("[![docs.rs](https://img.shields.io/docsrs/", env!("CARGO_PKG_NAME"), "?style=flat-square&logo=docs.rs)](https://docs.rs/", env!("CARGO_PKG_NAME"), ")")]
#![doc = "![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)"]
#![doc = concat!("![msrv](https://img.shields.io/badge/msrv-", env!("CARGO_PKG_RUST_VERSION"), "-blue?style=flat-square&logo=rust)")]
//! [![github](https://img.shields.io/github/stars/nik-rev/serde-cursor)](https://github.com/nik-rev/serde-cursor)
//!
//! This crate has a macro that takes a jq-like query as an argument and returns a type implementing [`Deserialize`].
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
//! ```
//! # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from("workspace = { package = { version = '0' } }")) } }
//! use serde_cursor::Cursor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = fs::read_to_string("Cargo.toml")?;
//!
//! let version: String = toml::from_str::<Cursor!(workspace.package.version)>(&data)?.0;
//! # Ok(()) }
//! ```
//!
//! **Without `serde_cursor`**:
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
//! ```
//! # mod fs { pub fn read_to_string(_: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(String::from("package = [{ name = '' }]")) } }
//! use serde_cursor::Cursor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = fs::read_to_string("Cargo.lock")?;
//!
//! let packages: Vec<String> = toml::from_str::<Cursor!(package.*.name)>(&file)?.0;
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
//! If `feature = "serde_with"` is enabled, [`struct@Cursor`] will implement [`serde_with::DeserializeAs`], meaning you can use it with `#[serde_as]`:
//!
//! ```
//! # use serde_with::serde_as;
//! use serde::{Serialize, Deserialize};
//! use serde_cursor::Cursor;
//!
//! #[serde_as]
//! #[derive(Serialize, Deserialize)]
//! struct CargoToml {
//!     #[serde_as(as = "Cursor!(workspace.package.version)")]
//!     version: String,
//! }
//! ```

mod de;
mod path_segment;
mod ser;

pub use path_segment::{ConstPathSegment, FieldName, Index, PathSegment};

use core::fmt;
use core::marker::PhantomData;

/// The [`Cursor!`] macro.
#[doc(inline)]
pub use serde_cursor_impl::Cursor;

/// Type returned by the [`Cursor!`] macro.
pub struct Cursor<T, P>(pub T, #[doc(hidden)] PhantomData<P>);

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

pub use const_str::C1;
pub use const_str::C2;
pub use const_str::C3;
pub use const_str::C4;
pub use const_str::ConstStr;
pub use const_str::StrLen;

mod const_str;

pub struct Nil;

pub struct Cons<S, P>(PhantomData<(S, P)>);

pub struct Wildcard;

pub trait Sequence: Default {
    type Item;
    fn push(&mut self, item: Self::Item);
}

impl<T> Sequence for Vec<T> {
    type Item = T;
    fn push(&mut self, item: Self::Item) {
        self.push(item);
    }
}
