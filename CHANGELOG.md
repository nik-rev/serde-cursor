# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

[Unreleased]: https://github.com/nik-rev/serde-cursor/compare/v0.3.0...HEAD

## [v0.3.0] - 2026-03-22

[v0.3.0]: https://github.com/nik-rev/serde-cursor/compare/v0.2.1...v0.3.0

### Added

You can now use interpolations in cursor paths, to factor out repetitive queries:

```rust
type Details<RestOfPath> = CursorPath!(properties.timeseries.*.data.instant.details + RestOfPath);

let pressure: Vec<f64> = toml::from_str::<Cursor!(france.$Details.air_pressure_at_sea_level)>(france)?.0;
let humidity: Vec<f64> = toml::from_str::<Cursor!(japan.$Details.relative_humidity)>(japan)?.0;
let temperature: Vec<f64> = toml::from_str::<Cursor!(japan.$Details.air_temperature)>(japan)?.0;
```

### Changed

Renamed some internal types that are **mostly implementation details**, but available in case you need to use them for some reason.

|old|new|
|---|---|
|`Nil`|`PathEnd`|
|`Cons`|`Path`|
|`FieldName`|`Field`|
|`Wildcard`|`IndexAll`|
|`SerializeCursor`|`SerializePath`|

## [v0.2.1] - 2026-03-21

[v0.2.1]: https://github.com/nik-rev/serde-cursor/compare/v0.2.0...v0.2.1

- Documentation improvements

## [v0.2.0] - 2026-03-21

[v0.2.0]: https://github.com/nik-rev/serde-cursor/compare/v0.1.5...v0.2.0

### Added

You can now use dashes in bare field names:

```rust
let version: String = toml::from_str::<Cursor!(dev-dependencies.serde.version)>(file)?.0;
```

## [v0.1.5] - 2026-03-20

[v0.1.5]: https://github.com/nik-rev/serde-cursor/compare/v0.1.4...v0.1.5

- Documentation improvements

## [v0.1.4] - 2026-03-20

[v0.1.4]: https://github.com/nik-rev/serde-cursor/compare/v0.1.3...v0.1.4

- Documentation improvements

## [v0.1.3] - 2026-03-20

[v0.1.3]: https://github.com/nik-rev/serde-cursor/compare/v0.1.2...v0.1.3

- Documentation improvements

## [v0.1.2] - 2026-03-20

[v0.1.2]: https://github.com/nik-rev/serde-cursor/compare/v0.1.1...v0.1.2

- Documentation improvements

## [v0.1.1] - 2026-03-20

[v0.1.1]: https://github.com/nik-rev/docstr/compare/v0.1.0...v0.1.1

- Documentation improvements
