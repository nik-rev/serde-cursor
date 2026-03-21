# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

[Unreleased]: https://github.com/nik-rev/serde-cursor/compare/v0.2.1...HEAD

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
