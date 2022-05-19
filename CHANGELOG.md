# Changelog



## [0.4.7](https://github.com/Blobfolio/dowser/releases/tag/v0.4.7) - 2022-05-18

### Changed

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
