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

This crate allows you to declaratively specify how to fetch the desired parts of a serde-compatible data format (such as JSON)
efficiently, without loading it all into memory, using a jq-like language.

```toml
serde_cursor = "0.3"
```

## Examples

The `Cursor!` macro makes it extremely easy to extract nested fields from data.

### Get version from `Cargo.toml`

```rust
use serde_cursor::Cursor;

let data = r#"
    [workspace.package]
    version = "0.1"
"#;

let version: String = toml::from_str::<Cursor!(workspace.package.version)>(data)?.0;
assert_eq!(version, "0.1");
```

`Cursor!(workspace.package.version)` is the magic juice - this type-macro expands to a type that implements [`serde::Deserialize`](https://docs.rs/serde_core/1.0.228/serde_core/de/trait.Deserialize.html).

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

let data = r#"
    [workspace.package]
    version = "0.1"
"#;

let version = toml::from_str::<CargoToml>(data)?.workspace.package.version;
```

### Get names of all dependencies from `Cargo.lock`

The index-all `.*` accesses every element in an array:

```rust
use serde_cursor::Cursor;

let file = r#"
    [[package]]
    name = "serde"

    [[package]]
    name = "rand"
"#;

let packages: Vec<String> = toml::from_str::<Cursor!(package.*.name)>(file)?.0;

assert_eq!(packages, vec!["serde", "rand"]);
```

## Syntax

Specify the type `Vec<String>` after the path `package.*.name`:

```rust
let packages = toml::from_str::<Cursor!(package.*.name: Vec<String>)>(file)?.0;
```

The type can be omitted, in which case it will be inferred:

```rust
let packages: Vec<String> = toml::from_str::<Cursor!(package.*.name)>(file)?.0;
```

Fields that consist of identifiers and `-`s can be used without quotes:

```rust
Cursor!(dev-dependencies.serde.version)
```

Fields that contain spaces or other special characters must be quoted:

```rust
Cursor!(ferris."🦀::<>".r#"""#)
```

You can access specific elements of an array:

```rust
Cursor!(package.0.name)
```

### Interpolations

It’s not uncommon for multiple queries to get quite repetitive:

```rust
let pressure: Vec<f64> = toml::from_str::<Cursor!(france.properties.timeseries.*.data.instant.details.air_pressure_at_sea_level)>(france)?.0;
let humidity: Vec<f64> = toml::from_str::<Cursor!(japan.properties.timeseries.*.data.instant.details.relative_humidity)>(japan)?.0;
let temperature: Vec<f64> = toml::from_str::<Cursor!(japan.properties.timeseries.*.data.instant.details.air_temperature)>(japan)?.0;
```

`serde_cursor` supports **interpolations**. You can factor out a common path into a type `Details`, and then interpolate it with `$Details` in the path inside `Cursor!`:

```rust
type Details<RestOfPath> = serde_cursor::Path!(properties.timeseries.*.data.instant.details + RestOfPath);

let pressure: Vec<f64> = toml::from_str::<Cursor!(france.$Details.air_pressure_at_sea_level)>(france)?.0;
let humidity: Vec<f64> = toml::from_str::<Cursor!(japan.$Details.relative_humidity)>(japan)?.0;
let temperature: Vec<f64> = toml::from_str::<Cursor!(japan.$Details.air_temperature)>(japan)?.0;
```

## `serde_cursor` vs [`serde_query`](https://github.com/pandaman64/serde-query)

`serde_query` also implements jq-like queries, but more verbosely.

### Single query

`serde_cursor`:

```rust
use serde_cursor::Cursor;

let data = r#"{ "commits": [{"author": "Ferris"}] }"#;

let authors: Vec<String> = serde_json::from_str::<Cursor!(commits.*.author)>(data)?.0;
```

`serde_query`:

```rust
use serde_query::Deserialize;

#[derive(Deserialize)]
struct Data {
    #[query(".commits.[].author")]
    authors: Vec<String>,
}

let data = r#"{ "commits": [{"author": "Ferris"}] }"#;
let data: Data = serde_json::from_str(data)?;

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

let data = r#"{ "count": 1, "commits": [{"author": "Ferris"}] }"#;

let data: Data = serde_json::from_str(data)?;
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

let data = r#"{ "count": 1, "commits": [{"author": "Ferris"}] }"#;

let data: Data = serde_json::from_str(data)?;
```

## Great error messages

When deserialization fails, you get the exact path of where the failure occurred:

```rust
use serde_cursor::Cursor;

let data = serde_json::json!({ "author": { "id": "not-a-number" } });
let result = serde_json::from_value::<Cursor!(author.id: i32)>(data);
let err = result.unwrap_err().to_string();
assert_eq!(err, r#".author.id: invalid type: string "not-a-number", expected i32"#);
```

## `serde_with` integration

If `feature = "serde_with"` is enabled, the type returned by `Cursor!` will implement [`serde_with::DeserializeAs`](https://docs.rs/serde_with/3.18.0/serde_with/de/trait.DeserializeAs.html) and [`serde_with::SerializeAs`](https://docs.rs/serde_with/3.18.0/serde_with/ser/trait.SerializeAs.html),
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

## How does it work?

The `Cursor!` macro expands to a recursive type that implements [`serde::Deserialize`](https://docs.rs/serde_core/1.0.228/serde_core/de/trait.Deserialize.html).
Information on how to access the nested fields is stored entirely inside the type system.

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

That macro is expanded into a `Cursor` type, which implements [`serde::Deserialize`](https://docs.rs/serde_core/1.0.228/serde_core/de/trait.Deserialize.html) and [`serde::Serialize`](https://docs.rs/serde_core/1.0.228/serde_core/ser/trait.Serialize.html):

```rust
Cursor<
    String, // : String
    Path<
        Field<"package">, // .package
        Path<
            IndexAll, // .*
            Path<
                Field<"dependencies">, // .dependencies
                Path<
                    Index<0>, // .0
                    PathEnd
                >,
            >,
        >,
    >,
>
```

The above is essentially an equivalent to:

```rust
vec![
    Segment::Field("package"), // .package
    Segment::IndexAll, // .*
    Segment::Field("dependencies"), // .dependencies
    Segment::Index(0) // .0
]
```

Except it exists entirely in the type system.

Each time the [`serde::Deserialize::deserialize()`](https://docs.rs/serde/latest/serde/trait.Deserialize.html#tymethod.deserialize) function is called,
the first segment of the path (`.package`) is processed, and the rest of the path (`.*.dependencies.0`) is passed to the
[`serde::Deserialize`](https://docs.rs/serde_core/1.0.228/serde_core/de/trait.Deserialize.html) trait, again, and again - until the path is empty.

Once the path is empty, we finally get to the type of the field - the `String` in the above example,
and finally call [`serde::Deserialize::deserialize()`](https://docs.rs/serde/latest/serde/trait.Deserialize.html#tymethod.deserialize) on that, to finish things off -
this `String` is then bubbled up the stack and returned from `<Cursor<String, _> as serde::Deserialize>::deserialize`.

<!-- cargo-reedme: end -->
