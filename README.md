# Dowser

[![docs.rs](https://img.shields.io/docsrs/dowser.svg?style=flat-square&label=docs.rs)](https://docs.rs/dowser/)
[![changelog](https://img.shields.io/crates/v/dowser.svg?style=flat-square&label=changelog&color=9b59b6)](https://github.com/Blobfolio/dowser/blob/master/CHANGELOG.md)<br>
[![crates.io](https://img.shields.io/crates/v/dowser.svg?style=flat-square&label=crates.io)](https://crates.io/crates/dowser)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/dowser/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/dowser/actions)
[![deps.rs](https://deps.rs/crate/dowser/latest/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/crate/dowser/)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/dowser/issues)



`Dowser` is a(nother) fast, recursive file-finding library for Rust, but it differs from [`walkdir`](https://crates.io/crates/walkdir) and kin in a number of ways:

* It is not limited to one root; any number of file and directory paths can be loaded and traversed en masse;
* Symlinks are followed by default, but can be disabled using `Dowser::without_symlinks`;
* Hidden paths and mount points are traversed like anything else;
* Matching file paths are canonicalized and deduped before yielding;
* Directory paths are automatically crawled but **not** yielded;



## Example

Create a new instance using `Dowser::default`, then specify root paths to ignore and/or include with `Dowser::without_path` and `Dowser::with_path`, respectively.

From there, leverage your favorite [`Iterator`](std::iter::Iterator) trait methods to filter/collect the results.

```rust
use dowser::Dowser;
use std::path::PathBuf;

// Return all files under "/usr/share/man", and probably some from other places
// since some programs prefer to symlink their documentation.
let men: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .collect();

// Same as above, but filtering paths as discovered so as to only keep the
// gzipped ones.
let men_gz: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .filter(|p|
        p.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    )
    .collect();
```



## Example

All you need to do is chain `Dowser::default` with one or more of the following seed methods:

* `Dowser::with_path` / `Dowser::with_paths`
* `Dowser::without_path` / `Dowser::without_paths`

From there, you can apply any `Iterator` methods you want.

```rust
use dowser::Dowser;
use std::path::PathBuf;

// Return all files under "/usr/share/man".
let files1: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .collect();

// Return only Gzipped files using callback filter.
let files1: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .filter(|p|
        p.extension().is_some_and(|e| e.eq_ignore_ascii_case("gz"))
    )
    .collect();
```



## Installation

Add `dowser` to your `dependencies` in `Cargo.toml`, like:

```toml
[dependencies]
dowser = "0.17.*"
```
