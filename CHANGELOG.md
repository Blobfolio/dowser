# Changelog



## [0.3.3](https://github.com/Blobfolio/dowser/releases/tag/v0.3.3) - 2021-12-20

### Added

* `dowser::Dowser::without_paths`
* `dowser::Dowser::without_path`



## [0.3.2](https://github.com/Blobfolio/dowser/releases/tag/v0.3.2) - 2021-12-15

### Added

* `dowser::Dowser::into_vec`
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
