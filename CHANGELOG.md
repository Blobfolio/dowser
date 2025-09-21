# Changelog



## [0.17.1](https://github.com/Blobfolio/dowser/releases/tag/v0.17.1) - 2025-09-21

### Changed

* Miscellaneous code cleanup and lints



## [0.17.0](https://github.com/Blobfolio/dowser/releases/tag/v0.17.0) - 2025-09-18

### New

* `Dowser::From<OsString>`/`From<&OsString>`
* `Dowser::From<Vec<T>>`/`From<&[T]>` (for all `From<T>` types)

### Changed

* `Iterator::size_hint` no longer provides upper bound
* Miscellaneous code cleanup and lints

### Breaking

* Rename `Dowser::read_paths_from_file` to `Dowser::push_paths_from_file`
* Remove `Dowser::with_paths`
* Remove `Dowser::without_paths`



## [0.16.1](https://github.com/Blobfolio/dowser/releases/tag/v0.16.1) - 2025-08-21

### Fixed

* `Dowser::without_paths` queued paths for crawl instead of ignoring them.



## [0.16.0](https://github.com/Blobfolio/dowser/releases/tag/v0.16.0) - 2025-07-07

### New

* `Extension::as_bytes`
* `Extension::as_str`
* `Extension::from_path_slice`
* `Extension::from_path`
* `Extension::is_empty`
* `Extension::len`
* `Extension::matches_path_slice`
* `Extension::matches_path`
* `Extension::new`
* Impl `AsRef<[u8]>` for `Extension`
* Impl `AsRef<str>` for `Extension`
* Impl `Ord`/`PartialOrd` for `Extension` (length, then value)

### Changed

* `Extension` now supports lengths of `1..=8` (instead of just `2..=4`)
* `Extension` now supports `!`, `#`, `+`, `-`, and `_` (in addition to ASCII alphanumerics)

### Breaking

* `Extension` is now a struct (instead of an enum)
* `Extension` methods are no longer differentiated by length:
  * Replaced `Extension::new2`/`new3`/`new4` w/ `Extension::new`
  * Replaced `Extension::try_from2`/`try_from3`/`try_from4` w/ `Extension::from_path`
  * Removed `Extension::slice_ext2`/`slice_ext3`/`slice_ext4` (use `std::path::Path::extension` instead)
* `Hash` for `Extension` now uses `Hasher::write_u64` for all values
* Removed `PartialEq<AsRef<Path>>` for `Extension` (use `Extension::matches_path` instead)
* Removed `Extension::codegen` (use `const {}` assignments instead)
* Removed `Extension::slice_ext` (use `std::path::Path::extension` instead)



## [0.15.0](https://github.com/Blobfolio/dowser/releases/tag/v0.15.0) - 2025-06-26

### Changed

* Bump `brunch` to `0.11` (dev)
* Bump `dactyl` to `0.13`
* Bump MSRV to `1.88`
* Miscellaneous code cleanup and lints



## [0.14.1](https://github.com/Blobfolio/dowser/releases/tag/v0.14.1) - 2025-06-01

### Changed

* Bump `dactyl` to `0.12`



## [0.14.0](https://github.com/Blobfolio/dowser/releases/tag/v0.14.0) - 2025-05-29

### Changed

* Bump `dactyl` to `0.11`
* Bump MSRV to `1.87`
* Miscellaneous code cleanup and lints

Most of the work this cycle relates to `Extension`:

* `Extension::new2`/`Extension::new3`/`Extension::new4` now `debug_assert!` to help draw attention to incorrect usage
* Explicitly require extensions be ASCII alphanumeric
* Derive `Eq`/`PartialEq` (instead of implementing manually) so constants can be used as `matches!` patterns
* `Extension::slice_ext` is now `const`
* Change `Debug` impl to render inner value as human-readable string
* Implement `Display`



## [0.13.0](https://github.com/Blobfolio/dowser/releases/tag/v0.13.0) - 2025-05-15

### New

* `Dowser::push_path`

### Changed

* Bump `brunch` to `0.10` (dev)



## [0.12.0](https://github.com/Blobfolio/dowser/releases/tag/v0.12.0) - 2025-02-25

### New

* `Dowser::without_symlinks`

### Changed

* Bump `brunch` to `0.9` (dev)
* Bump `dactyl` to `0.10`
* Bump MSRV to `1.85`
* Bump Rust edition to `2024`

### Removed

* `Dowser::into_vec` (use `collect` instead)
* `Dowser::into_vec_filtered` (use `filter`+`collect` instead)



## [0.11.0](https://github.com/Blobfolio/dowser/releases/tag/v0.11.0) - 2024-12-10

### Changed

* Bump `brunch` to `0.8` (dev)
* Bump `dactyl` to `0.9`
* Bump MSRV to `1.83`



## [0.10.1](https://github.com/Blobfolio/dowser/releases/tag/v0.10.1) - 2024-11-28

### Changed

* Bump `brunch` to `0.7`
* Bump `dactyl` to `0.8`
* Miscellaneous code cleanup and lints



## [0.10.0](https://github.com/Blobfolio/dowser/releases/tag/v0.10.0) - 2024-10-22

### New

* `Dowser::read_paths_from_file`

### Changed

* Bump MSRV to `1.81`
* Miscellaneous code cleanup and lints



## [0.9.3](https://github.com/Blobfolio/dowser/releases/tag/v0.9.3) - 2024-09-05

### Changed

* Miscellaneous code cleanup and lints
* Bump `brunch` to `0.6`



## [0.9.2](https://github.com/Blobfolio/dowser/releases/tag/v0.9.2) - 2024-07-25

### Changed

* Miscellaneous code lints



## [0.9.1](https://github.com/Blobfolio/dowser/releases/tag/v0.9.1) - 2024-06-13

### Changed

* Add a few `#[inline]` hints to improve downstream performance



## [0.9.0](https://github.com/Blobfolio/dowser/releases/tag/v0.9.0) - 2024-02-15

### Changed

* Bump MSRV to `1.72`



## [0.8.2](https://github.com/Blobfolio/dowser/releases/tag/v0.8.2) - 2024-02-08

### Changed

* Bump `dactyl` to `0.7`
* Minor doc/script cleanup



## [0.8.1](https://github.com/Blobfolio/dowser/releases/tag/v0.8.1) - 2023-10-15

### Changed

* Bump `dactyl` to `0.6`



## [0.8.0](https://github.com/Blobfolio/dowser/releases/tag/v0.8.0) - 2023-06-01

### Changed

* Bump MSRV `1.70`
* Update dependencies
* Remove all `unsafe` code
* Improve unit test coverage
* Minor code changes and lints



## [0.7.0](https://github.com/Blobfolio/dowser/releases/tag/v0.7.0) - 2023-01-27

### New

* `Extension::slice_ext2`
* `Extension::slice_ext3`
* `Extension::slice_ext4`

### Changed

* `Extension` is now exposed for non-Unix targets

### Removed

* `utility::path_as_bytes`



## [0.6.4](https://github.com/Blobfolio/dowser/releases/tag/v0.6.4) - 2023-01-26

### Changed

* Bump brunch `0.4`
* Fix ci badge (docs)



## [0.6.3](https://github.com/Blobfolio/dowser/releases/tag/v0.6.3) - 2022-11-03

### Changed

* Relax `ahash` version requirements



## [0.6.2](https://github.com/Blobfolio/dowser/releases/tag/v0.6.2) - 2022-09-22

### Changed

* Bump MSRV `1.63`
* Improved docs



## [0.6.1](https://github.com/Blobfolio/dowser/releases/tag/v0.6.1) - 2022-09-09

### Cleanup

* Drop optional `rayon` from `Cargo.toml`
* Drop unused `ahash` features



## [0.6.0](https://github.com/Blobfolio/dowser/releases/tag/v0.6.0) - 2022-09-09

### New

* `Dowser::into_vec_filtered` (_f._ `Dowser::into_vec`)
* `Dowser::into_vec`

### Changed

* Traversal is now fully serial. (Multi-threading came with too many gotchas and didn't improve performance much for most workloads.)
* `Dowser::with_paths` and `Dowser::without_paths` — the plural methods — will now explicitly panic if passed a direct `Path` or `PathBuf` instead of a _proper_ `Iterator` of paths.
* `Dowser` now has basic Windows support.



## [0.5.3](https://github.com/Blobfolio/dowser/releases/tag/v0.5.3) - 2022-08-11

### Changed

* Bump ahash `0.8.0`



## [0.5.2](https://github.com/Blobfolio/dowser/releases/tag/v0.5.2) - 2022-06-18

### Misc

* Update dependencies.



## [0.5.1](https://github.com/Blobfolio/dowser/releases/tag/v0.5.1) - 2022-05-27

### Fixed

* Files could be erroneously skipped when crossing filesystem boundaries

### Removed

* Feature `parking_lot_mutex` (`std::sync::Mutex` is faster as of Rust `1.62`)



## [0.5.0](https://github.com/Blobfolio/dowser/releases/tag/v0.5.0) - 2022-05-27

This release removes `DirConcurrency` and related methods.

Parallel directory reads are now automatic and mandatory, but the inner loops — reading/filtering the contents of those directories — are now executed serially (within each parallel thread), greatly reducing the number of concurrently open file handles and subsequent risk of hitting `ulimit` ceilings.

The file collision (uniqueness filters) have also been greatly improved, further reducing the number of syscalls and overall search times.



## [0.4.7](https://github.com/Blobfolio/dowser/releases/tag/v0.4.7) - 2022-05-18

### Changed

* Lock third-party dependency versions
* Faster parallel iteration
* Lower `DirConcurrency::Sane` from `threads - 1` to `threads / 2`



## [0.4.6](https://github.com/Blobfolio/dowser/releases/tag/v0.4.6) - 2022-04-16

### Added

* `Extension::codegen` (compile-time helper)
* `Extension::slice_ext`



## [0.4.5](https://github.com/Blobfolio/dowser/releases/tag/v0.4.5) - 2022-03-29

### Changed

* Replace hasher with `dactyl::NoHash`



## [0.4.4](https://github.com/Blobfolio/dowser/releases/tag/v0.4.4) - 2022-03-27

### Added

* impl `From<&OsStr>`
* impl `From<&str>`
* impl `From<&String>`
* impl `From<String>`

### Deprecated

* `DirConcurrency::Other` (prefer `DirConcurrency::Custom`)

### Changed

* `DirConcurrency::Single` now does all processing in serial



## [0.4.3](https://github.com/Blobfolio/dowser/releases/tag/v0.4.3) - 2022-03-26

### Added

* impl `Clone` for `Dowser`
* `Dowser::into_vec`
* `Dowser::with_dir_concurrency`



## [0.4.2](https://github.com/Blobfolio/dowser/releases/tag/v0.4.2) - 2022-03-08

### Changed

* Minor performance improvements.



## [0.4.0](https://github.com/Blobfolio/dowser/releases/tag/v0.4.0) - 2022-03-07

This release contains breaking changes:

`dowser::Dowser` has been refactored into a proper `Iterator<Item=PathBuf>`. The struct-specific filters and `regexp` crate feature have been removed.

This version is slightly slower than `0.3.x`, but should be a lot more flexible, while also being less likely to run into `ulimit` system caps.

### Added

* impl `Hash` for `Extension`

### Removed

* `dowser::dowse`
* `utility::du`



## [0.3.6](https://github.com/Blobfolio/dowser/releases/tag/v0.3.6) - 2022-01-29

### Changed

* Update dependencies;
* Fix feature-dependent doctests;
* Make `parking_lot` dependency optional (but still default);
* Replace `flume` with `crossbeam-channel`;

### Deprecated

* `utility::du`



## [0.3.5](https://github.com/Blobfolio/dowser/releases/tag/v0.3.5) - 2021-12-30

### Added

* `Dowser::with_capacity`
* `Dowser::with_capacity_and_filter`
* `Dowser::shallow`

### Changed

* Use `parking_lot` and `flume` for slightly faster processing.



## [0.3.4](https://github.com/Blobfolio/dowser/releases/tag/v0.3.4) - 2021-12-21

### Added

* `Dowser::par_without_paths`

### Improved

* Documentation.


## [0.3.3](https://github.com/Blobfolio/dowser/releases/tag/v0.3.3) - 2021-12-20

### Added

* `Dowser::without_paths`
* `Dowser::without_path`



## [0.3.2](https://github.com/Blobfolio/dowser/releases/tag/v0.3.2) - 2021-12-15

### Added

* `Dowser::into_vec`
* `From<&OsStr>`
* `From<&OsString>`
* `From<&Path>`
* `From<&PathBuf>`
* `From<&str>`
* `From<&String>`
* `From<[PathBuf; 1..=32]>`
* `From<HashSet<PathBuf>>`
* `From<OsString>`
* `From<PathBuf>`
* `From<String>`
* `From<Vec<PathBuf>>`

### Improved

* Path deduplication.



## [0.3.1](https://github.com/Blobfolio/dowser/releases/tag/v0.3.1) - 2021-12-14

### Deprecated

* `dowser::dowse` has been deprecated; use `dowser::Dowser::default()` instead.



## [0.3.0](https://github.com/Blobfolio/dowser/releases/tag/v0.3.0) - 2021-10-21

### Added

* This changelog! Haha.

### Changed

* Use Rust edition 2021.
