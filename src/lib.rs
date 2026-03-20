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
//! # mod fs { fn read_to_string(_: &str) -> String { String::new() } }
//! use serde_cursor::Cursor;
//!
//! let data = fs::read_to_string("Cargo.toml")?;
//!
//! let version: String = toml::from_str::<Cursor!(workspace.package.version)>(&data)?.0;
//! ```
//!
//! **Without `serde_cursor`**:
//!
//! ```
//! # mod fs { fn read_to_string(_: &str) -> String { String::new() } }
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
//! struct Workspace {
//!     version: String
//! }
//!
//! let data = fs::read_to_string("Cargo.toml")?;
//!
//! let version = toml::from_str::<CargoToml>(&data)?.workspace.package.version;
//! ```
//!
//! ## Get all dependencies from `Cargo.lock`
//!
//! ```
//! # mod fs { fn read_to_string(_: &str) -> String { String::new() } }
//! use serde_cursor::Cursor;
//!
//! let file = fs::read_to_string("Cargo.lock")?;
//!
//! let packages: Vec<String> = toml::from_str::<Cursor!(package.*.name)>(&file)?.0;
//! ```
//!
//! **Without `serde_cursor`**:
//!
//! ```
//! # mod fs { fn read_to_string(_: &str) -> String { String::new() } }
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
//! let file = fs::read_to_string("Cargo.lock")?;
//!
//! let packages = toml::from_str::<CargoLock>(&file)?
//!     .package
//!     .into_iter()
//!     .map(|pkg| pkg.name)
//!     .collect::<Vec<_>>();
//! ```
//!
//! # `serde_cursor` vs [`serde_query`]
//!
//! `serde_query` is significantly more verbose.
//!
//! ## Single query
//!
//! `serde_cursor`:
//!
//! ```
//! # mod fs { fn read_to_string(_: &str) -> String { String::new() } }
//! use serde_cursor::Cursor;
//!
//! let data = fs::read_to_string("data.json")?;
//!
//! let authors: Vec<String> = serde_json::from_str::<Cursor!(commits.*.author)>(&data)?.0;
//! ```
//!
//! `serde_query`:
//!
//! ```
//! # mod fs { fn read_to_string(_: &str) -> String { String::new() } }
//! use serde_query::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Data {
//!     #[query(".commits.[].author")]
//!     authors: Vec<String>,
//! }
//!
//! let data = fs::read_to_string("data.json")?;
//! let data: Data = serde_json::from_str(&data)?;
//!
//! let authors = data.authors;
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
//!     authors: Cursor!(commits.*.author: Vec<String>),
//!     count: Cursor!(count: usize),
//! }
//!
//! let data = fs::read_to_string("data.json")?;
//!
//! let data: Data = serde_json::from_str(&data)?;
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
//! let data = fs::read_to_string("data.json")?;
//!
//! let data: Data = serde_json::from_str(&data)?;
//! ```
//!
//! # `serde_with` integration
//!
//! If `feature = "serde_with"` is enabled, [`struct@Cursor`] will implement [`serde_with::DeserializeAs`], meaning you can use it with `#[serde_as]`:
//!
//! ```
//! # use serde_as::serde_as;
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

use core::fmt;
use core::marker::PhantomData;
use serde_core::de::{
    Deserialize, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor,
};

/// The [`Cursor!`] macro.
#[doc(inline)]
pub use serde_cursor_impl::Cursor;

/// Type returned by the [`Cursor!`] macro.
pub struct Cursor<T, P>(pub T, #[doc(hidden)] PhantomData<P>);

impl<'de, T, P> Deserialize<'de> for Cursor<T, P>
where
    T: Deserialize<'de>,
    P: Path<'de, T>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = P::navigate(deserializer)?;
        Ok(Self(value, PhantomData))
    }
}

#[doc(hidden)]
pub use const_str::C1;
#[doc(hidden)]
pub use const_str::C2;
#[doc(hidden)]
pub use const_str::C3;
#[doc(hidden)]
pub use const_str::C4;
#[doc(hidden)]
pub use const_str::ConstStr;
#[doc(hidden)]
pub use const_str::StrLen;

mod const_str;

#[doc(hidden)]
pub enum PathSegment {
    Field(&'static str),
    Index(usize),
}

#[doc(hidden)]
pub trait ConstPathSegment {
    const VALUE: PathSegment;
}

#[doc(hidden)]
pub struct Nil;
#[doc(hidden)]
pub struct Cons<S, P>(PhantomData<(S, P)>);
#[doc(hidden)]
pub struct Wildcard;

struct WildcardVisitor<P, C> {
    _marker: PhantomData<(P, C)>,
}

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

impl<'de, P, C> Visitor<'de> for WildcardVisitor<P, C>
where
    C: Sequence,
    P: Path<'de, C::Item>,
    C::Item: Deserialize<'de>,
{
    type Value = C;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut items = C::default();
        while let Some(item) = seq.next_element_seed(PathSeed::<P, C::Item>(PhantomData))? {
            items.push(item);
        }
        Ok(items)
    }
}

impl<'de, P, C> Path<'de, C> for Cons<Wildcard, P>
where
    C: Sequence,
    P: Path<'de, C::Item>,
    C::Item: Deserialize<'de>,
{
    fn navigate<D>(deserializer: D) -> Result<C, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(WildcardVisitor::<P, C> {
            _marker: PhantomData,
        })
    }
}

#[diagnostic::on_unimplemented(
    message = "`{T}` doesn't implement `serde_cursor::Sequence`",
    note = "try: `Vec<{T}>`"
)]
#[doc(hidden)]
pub trait Path<'de, T> {
    fn navigate<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>;
}

// base case: we are at the target property
impl<'de, T: Deserialize<'de>> Path<'de, T> for Nil {
    fn navigate<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer)
    }
}

// step case: we are still digging into the object
impl<'de, S, P, T> Path<'de, T> for Cons<S, P>
where
    S: ConstPathSegment,
    P: Path<'de, T>,
    T: Deserialize<'de>,
{
    fn navigate<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        match S::VALUE {
            PathSegment::Field(name) => deserializer.deserialize_map(FieldVisitor::<P, T> {
                target: name,
                _marker: PhantomData,
            }),
            PathSegment::Index(index) => deserializer.deserialize_seq(SequenceVisitor::<P, T> {
                target_index: index,
                _marker: PhantomData,
            }),
        }
    }
}

struct SequenceFieldVisitor<P, T, V> {
    target: &'static str,
    _marker: PhantomData<(P, T, V)>,
}

impl<'de, P, T, V> Visitor<'de> for SequenceFieldVisitor<P, T, V>
where
    P: Path<'de, T>,
    T: Deserialize<'de>,
    V: Seq<Item = T>,
{
    type Value = V;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = V::default();

        while let Some(value) =
            seq.next_element_seed(PathSeed2::<P, T>(self.target, PhantomData))?
        {
            V::push(&mut values, value);
        }

        Ok(values)
    }
}

trait Seq: Default {
    type Item;

    fn push(&mut self, value: Self::Item);
}

impl<'de, T: Deserialize<'de>> Seq for Vec<T> {
    type Item = T;

    fn push(&mut self, value: Self::Item) {
        Vec::push(self, value);
    }
}

struct SequenceVisitor<P, T> {
    target_index: usize,
    _marker: PhantomData<(P, T)>,
}

impl<'de, P, T> Visitor<'de> for SequenceVisitor<P, T>
where
    P: Path<'de, T>,
    T: Deserialize<'de>,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "a sequence containing at least {} elements",
            self.target_index + 1
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // skip elements before the target index
        for i in 0..self.target_index {
            if seq.next_element::<IgnoredAny>()?.is_none() {
                return Err(serde_core::de::Error::custom(format!(
                    "index {} out of bounds (length {})",
                    self.target_index, i
                )));
            }
        }

        // found the index, recurse to the next part of the path
        let result = seq
            .next_element_seed(PathSeed::<P, T>(PhantomData))?
            .ok_or_else(|| {
                serde_core::de::Error::custom(format!("index {} out of bounds", self.target_index))
            })?;

        // consume the rest of the sequence
        // some deserializers (like serde_json) will error if the sequence isn't exhausted
        while seq.next_element::<IgnoredAny>()?.is_some() {}

        Ok(result)
    }
}

struct FieldVisitor<P, D> {
    target: &'static str,
    _marker: PhantomData<(P, D)>,
}

impl<'de, P, T> Visitor<'de> for FieldVisitor<P, T>
where
    P: Path<'de, T>,
    T: Deserialize<'de>,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "map with field '{}'", self.target)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result = None;

        while let Some(key) = map.next_key::<String>()? {
            if key == self.target && result.is_none() {
                result = Some(map.next_value_seed(PathSeed::<P, T>(PhantomData))?);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }

        result.ok_or_else(|| {
            serde_core::de::Error::custom(format!("field '{}' not found", self.target))
        })
    }
}

struct PathSeed2<P, T>(&'static str, PhantomData<(P, T)>);

impl<'de, P, T> DeserializeSeed<'de> for PathSeed2<P, T>
where
    P: Path<'de, T>,
    T: Deserialize<'de>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FieldVisitor::<P, T> {
            target: self.0,
            _marker: PhantomData,
        })
    }
}

struct PathSeed<P, T>(PhantomData<(P, T)>);

impl<'de, P, T> DeserializeSeed<'de> for PathSeed<P, T>
where
    P: Path<'de, T>,
    T: Deserialize<'de>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        P::navigate(deserializer)
    }
}

#[doc(hidden)]
pub struct FieldName<S: ConstStr>(PhantomData<S>);
#[doc(hidden)]
pub struct Index<const N: usize>;

#[doc(hidden)]
impl<S: ConstStr> ConstPathSegment for FieldName<S> {
    const VALUE: PathSegment = PathSegment::Field(S::VALUE);
}

#[doc(hidden)]
impl<const N: usize> ConstPathSegment for Index<N> {
    const VALUE: PathSegment = PathSegment::Index(N);
}
