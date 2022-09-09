/*!
# Dowser

[![Documentation](https://docs.rs/dowser/badge.svg)](https://docs.rs/dowser/)
[![Changelog](https://img.shields.io/crates/v/dowser.svg?label=Changelog&color=9cf)](https://github.com/Blobfolio/dowser/blob/master/CHANGELOG.md)
[![crates.io](https://img.shields.io/crates/v/dowser.svg)](https://crates.io/crates/dowser)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](https://github.com/Blobfolio/dowser)

[`Dowser`] is a(nother) fast, recursive file-finding library for Unix/Rust. It differs from [`Walkdir`](https://crates.io/crates/walkdir) and kin in a number of ways:

* It is not limited to one root; any number of file and directory paths can be loaded and traversed en masse;
* Symlinks and hidden directories are followed like any other, including across devices;
* Matching file paths are canonicalized and deduped before yielding;

If those things sound nice, this library might be a good fit.

On the other hand, [`Dowser`] is optimized for _file_ searching; the iterator crawls but does not yield directory paths.

Additionally, path deduping relies on Unix metadata; **this library is not compatible with Windows**;

Depending on your needs, those limitations could be bad, in which case something like [`Walkdir`](https://crates.io/crates/walkdir) would make more sense.



## Installation

Add `dowser` to your `dependencies` in `Cargo.toml`, like:

```text,ignore
[dependencies]
dowser = "0.6.*"
```



## Example

All you need to do is chain [`Dowser::default`] with one or more of the following seed methods:

* [`Dowser::with_path`] / [`Dowser::with_paths`]
* [`Dowser::without_path`] / [`Dowser::without_paths`]

From there, you can use whatever `Iterator` methods you want.

```
use dowser::{
    Dowser,
    Extension,
};
use std::path::PathBuf;

// Return all files under "/usr/share/man".
let files: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .collect();

// Return only Gzipped files using callback filter.
let files: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .filter(|p|
        p.extension().map_or(
            false,
            |e| e.eq_ignore_ascii_case("gz")
        )
    )
    .collect();

// Same Gzip example, but using Extension.
const EXT: Extension = Extension::new2(*b"gz");
let files: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .filter(|p| Some(EXT) == Extension::try_from2(p))
    .collect();
```
*/

#![deny(unsafe_code)]

#![warn(
	clippy::filetype_is_file,
	clippy::integer_division,
	clippy::integer_division,
	clippy::needless_borrow,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::suboptimal_flops,
	clippy::unneeded_field_pattern,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]
#![allow(clippy::module_name_repetitions)] // This is fine.



mod ext;
mod entry;
mod iter;
pub mod utility;



pub(crate) use entry::Entry;
pub use ext::Extension;
pub use iter::Dowser;
