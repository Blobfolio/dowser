# Changelog



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
