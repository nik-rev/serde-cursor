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
![msrv](https://img.shields.io/badge/msrv--blue?style=flat-square&logo=rust)
[![github](https://img.shields.io/github/stars/nik-rev/serde-cursor)](https://github.com/nik-rev/serde-cursor)

This crate has a macro that takes a jq-like query as an argument and returns a type implementing [`Deserialize`](https://docs.rs/serde_core/1.0.228/serde_core/de/trait.Deserialize.html).

```toml
serde_cursor = "0.1"
```

## Examples

The [`Cursor!`](https://docs.rs/serde_cursor_impl/latest/serde_cursor_impl/macro.Cursor.html) macro makes it extremely easy to extract nested fields from data.

### Get version from `Cargo.toml`

```rust
use serde_cursor::Cursor;

let data = fs::read_to_string("Cargo.toml")?;

let version: String = toml::from_str::<Cursor!(workspace.package.version)>(&data)?.0;
```

**Without `serde_cursor`**:

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
struct Workspace {
    version: String
}

let data = fs::read_to_string("Cargo.toml")?;

let version = toml::from_str::<CargoToml>(&data)?.workspace.package.version;
```

### Get all dependencies from `Cargo.lock`

```rust
use serde_cursor::Cursor;

let file = fs::read_to_string("Cargo.lock")?;

let packages: Vec<String> = toml::from_str::<Cursor!(package.*.name)>(&file)?.0;
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

## `serde_cursor` vs [`serde_query`]

`serde_query` is significantly more verbose.

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
    authors: Cursor!(commits.*.author: Vec<String>),
    count: Cursor!(count: usize),
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

If `feature = "serde_with"` is enabled, [`struct@Cursor`](https://docs.rs/serde_cursor/latest/serde_cursor/struct.Cursor.html) will implement [`serde_with::DeserializeAs`](https://docs.rs/serde_with/3.18.0/serde_with/de/trait.DeserializeAs.html), meaning you can use it with `#[serde_as]`:

```rust
use serde::{Serialize, Deserialize};
use serde_cursor::Cursor;

#[serde_as]
#[derive(Serialize, Deserialize)]
struct CargoToml {
    #[serde_as(as = "Cursor!(workspace.package.version)")]
    version: String,
}
```

<!-- cargo-reedme: end -->
