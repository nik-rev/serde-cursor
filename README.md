# `serde_cursor`

<!-- cargo-reedme: start -->

<!-- cargo-reedme: info-start

    Do not edit this region by hand
    ===============================

    This region was generated from Rust documentation comments by `cargo-reedme` using this command:

        cargo +nightly reedme

    for more info: https://github.com/nik-rev/cargo-reedme

cargo-reedme: info-end -->

[![crates.io](https://img.shields.io/crates/v/serde_cursor?style=flat-square&logo=rust)](https://crates.io/crates/serde_cursor)
[![docs.rs](https://img.shields.io/docsrs/serde_cursor?style=flat-square&logo=docs.rs)](https://docs.rs/serde_cursor)
![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)
![msrv](https://img.shields.io/badge/msrv-1.78-blue?style=flat-square&logo=rust)
[![github](https://img.shields.io/github/stars/nik-rev/serde-cursor)](https://github.com/nik-rev/serde-cursor)

This crate allows you to declaratively specify how to fetch the desired parts of a serde-compatible data format
efficiently, without loading it into memory, using a [jq](https://jqlang.org/tutorial/)-like language.

```toml
serde_cursor = "0.1"
```

## Examples

The [`Cursor!`](https://docs.rs/serde_cursor_impl/latest/serde_cursor_impl/macro.Cursor.html) macro makes it extremely easy to extract nested fields from data.

### Get version from `Cargo.toml`

```toml
# Cargo.toml
[workspace.package]
version = "0.1"
```

Accessed with `workspace.package.version`:

```rust
use serde_cursor::Cursor;

let data = fs::read_to_string("Cargo.toml")?;

let version: String = toml::from_str::<Cursor!(workspace.package.version)>(&data)?.0;
assert_eq!(version, "0.1");
```

**Without `serde_cursor`**:

*Pain and suffering…*

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct CargoToml {
    workspace: Workspace
}

#[derive(Deserialize)]
struct Workspace {
    package: Package
}

#[derive(Deserialize)]
struct Package {
    version: String
}

let data = fs::read_to_string("Cargo.toml")?;

let version = toml::from_str::<CargoToml>(&data)?.workspace.package.version;
```

### Get names of all dependencies from `Cargo.lock`

```toml
[[package]]
serde = "1.0"

[[package]]
rand = "0.9"
```

The wildcard `.*` accesses every element in an array:

```rust
use serde_cursor::Cursor;

let file = fs::read_to_string("Cargo.lock")?;

let packages: Vec<String> = toml::from_str::<Cursor!(package.*.name)>(&file)?.0;

assert_eq!(packages, vec!["serde", "rand"]);
```

**Without `serde_cursor`**:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct CargoLock {
    package: Vec<Package>
}

#[derive(Deserialize)]
struct Package {
    name: String
}

let file = fs::read_to_string("Cargo.lock")?;

let packages = toml::from_str::<CargoLock>(&file)?
    .package
    .into_iter()
    .map(|pkg| pkg.name)
    .collect::<Vec<_>>();
```

## `serde_cursor` vs [`serde_query`](https://github.com/pandaman64/serde-query)

`serde_query` also implements jq-like queries, but more verbosely.

### Single query

`serde_cursor`:

```rust
use serde_cursor::Cursor;

let data = fs::read_to_string("data.json")?;

let authors: Vec<String> = serde_json::from_str::<Cursor!(commits.*.author)>(&data)?.0;
```

`serde_query`:

```rust
use serde_query::Deserialize;

#[derive(Deserialize)]
struct Data {
    #[query(".commits.[].author")]
    authors: Vec<String>,
}

let data = fs::read_to_string("data.json")?;
let data: Data = serde_json::from_str(&data)?;

let authors = data.authors;
```

### Storing queries in a `struct`

`serde_cursor`:

```rust
use serde::Deserialize;
use serde_cursor::Cursor;

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "commits")]
    authors: Cursor!(*.author: Vec<String>),
    count: usize,
}

let data = fs::read_to_string("data.json")?;

let data: Data = serde_json::from_str(&data)?;
```

`serde_query`:

```rust
use serde_query::Deserialize;

#[derive(Deserialize)]
struct Data {
    #[query(".commits.[].author")]
    authors: Vec<String>,
    #[query(".count")]
    count: usize,
}

let data = fs::read_to_string("data.json")?;

let data: Data = serde_json::from_str(&data)?;
```

## `serde_with` integration

If `feature = "serde_with"` is enabled, [`Cursor`](https://docs.rs/serde_cursor/latest/serde_cursor/struct.Cursor.html) will implement [`serde_with::DeserializeAs`](https://docs.rs/serde_with/3.18.0/serde_with/de/trait.DeserializeAs.html) and [`serde_with::SerializeAs`](https://docs.rs/serde_with/3.18.0/serde_with/ser/trait.SerializeAs.html),
meaning you can use it with the `#[serde_as]` attribute:

```rust
use serde::{Serialize, Deserialize};
use serde_cursor::Cursor;

#[serde_as]
#[derive(Serialize, Deserialize)]
struct CargoToml {
    #[serde(rename = "workspace")]
    #[serde_as(as = "Cursor!(package.version)")]
    version: String,
}

let toml: CargoToml = toml::from_str("workspace = { package = { version = '0.1.0' } }")?;
assert_eq!(toml.version, "0.1.0");
assert_eq!(serde_json::to_string(&toml)?, r#"{"workspace":{"package":{"version":"0.1.0"}}}"#);
```

## Great error messages

When deserialization fails, you get the exact path of where the failure occurred.

```rust
use serde_cursor::Cursor;

let data = serde_json::json!({ "author": { "id": "not-a-number" } });
let result = serde_json::from_value::<Cursor!(author.id: i32)>(data);
let err = result.unwrap_err().to_string();
assert_eq!(err, r#".author.id: invalid type: string "not-a-number", expected i32"#);
```

## How does it work?

The [`Cursor!`](https://docs.rs/serde_cursor_impl/latest/serde_cursor_impl/macro.Cursor.html) macro is a “type-level” parser. It takes your jq-like query and transforms it into a nested, recursive type that implements [`serde::Deserialize`](https://docs.rs/serde_core/1.0.228/serde_core/de/trait.Deserialize.html).

Consider this query, which gets the first dependency of every dependency in `Cargo.toml`:

```rust
Cursor!(package.*.dependencies.0: String)
```

For this `Cargo.lock`, it would extract `["libc", "find-msvc-tools"]`:

```toml
[[package]]
name = "android_system_properties"
dependencies = ["libc"]

[[package]]
name = "cc"
dependencies = ["find-msvc-tools", "shlex"]
```

That macro is expanded into a [Cursor](https://docs.rs/serde_cursor/latest/serde_cursor/struct.Cursor.html) type, which implements [Deserialize](https://docs.rs/serde_core/1.0.228/serde_core/de/trait.Deserialize.html) and [Serialize](https://docs.rs/serde_core/1.0.228/serde_core/ser/trait.Serialize.html):

```rust
Cursor<
    String,
    Cons<
        Field<"package">,
        Cons<
            Wildcard,
            Cons<
                Field<"dependencies">,
                Cons<Index<0>, Nil>,
            >,
        >,
    >,
>
```

The above is essentially an equivalent to:

```rust
vec!["package", *, "dependencies", 0]
```

Except it exists entirely in the type system.

Each time the [`Deserialize::deserialize()`](https://docs.rs/serde/latest/serde/trait.Deserialize.html#tymethod.deserialize) function is called, the first element of the type-level list is removed,
and the rest of the list is passed to the [`Deserialize`](https://docs.rs/serde_core/1.0.228/serde_core/de/trait.Deserialize.html) trait, again.

This happens until the list is exhausted, in which case we finally get to the type of the field - the `String` in the above example,
and finally call [`Deserialize::deserialize()`](https://docs.rs/serde/latest/serde/trait.Deserialize.html#tymethod.deserialize) on that, to finish things off.

<!-- cargo-reedme: end -->
