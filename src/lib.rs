/*!
# Dowser

[![Documentation](https://docs.rs/dowser/badge.svg)](https://docs.rs/dowser/)
[![crates.io](https://img.shields.io/crates/v/dowser.svg)](https://crates.io/crates/dowser)

[`Dowser`] is a(nother) fast, multi-threaded, recursive file-finding library for Unix/Rust. It differs from [`Walkdir`](https://crates.io/crates/walkdir) and kin in a number of ways:

 * It is not limited to one root; any number of file and directory paths can be loaded and traversed en masse;
 * Symlinks and hidden directories are followed like any other, including across devices;
 * Matching file paths are canonicalized, deduped, and collected into a `Vec<PathBuf>`;

If those things sound nice, this library might be a good fit.

On the other hand, [`Dowser`] is optimized for just one particular type of searching:

 * File paths can be filtered via [`Dowser::filtered`] or [`Dowser::regex`], but directory paths cannot;
 * There are no settings for things like min/max depth, directory filtering, etc.;
 * It only returns *file* paths. Directories are crawled, but not returned in the set;
 * File uniqueness hashing relies on Unix metadata; **this library is not compatible with Windows**;

Depending on your needs, those limitations could be bad, in which case something like [`Walkdir`](https://crates.io/crates/walkdir) might make more sense.



## Installation

Add `dowser` to your `dependencies` in `Cargo.toml`, like:

```ignore
[dependencies]
dowser = "0.2.*"
```



## Features

| Feature | Description |
| ------- | ----------- |
| `regexp` | Enable the [`Dowser::regex`] method, which allows for matching file paths (as bytes) against a regular expression. |

To use this feature, alter the `Cargo.toml` bit to read:

```ignore
[dependencies.dowser]
version = "0.2.*"
features = [ "regexp" ]
```



## Example

This crate comes with two ways to find files. If you already have the full list of starting path(s) and just want *all the files* that exist under them, use the [`dowse`](self::dowse()) method:

```rust
use std::path::PathBuf;

let paths = [ "/path/one", "/path/two", "/path/three" ];
let files: Vec<PathBuf> = dowser::dowse(&paths);
```

If you want to filter files or need to add path(s) to the crawl list multiple times, initialize a [`Dowser`] object with one of the following three methods:

 * [`Dowser::default`]: Return all files without prejudice.
 * [`Dowser::filtered`]: Filter file paths via the provided callback.
 * [`Dowser::regex`]: Filter file paths via regular express. (This requires enabling the `regexp` crate feature.)

From there, add one or more file or directory paths using the [`Dowser::with_path`] and [`Dowser::with_paths`] methods.

Finally, collect the results with `Vec::<PathBuf>::try_from()`. If no files are found, an error is returned, otherwise the matching file paths are collected into a vector.

```rust
use dowser::Dowser;
use std::convert::TryFrom;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

// Return all files under "/usr/share/man".
let files = Vec::<PathBuf>::try_from(
   Dowser::default().with_path("/usr/share/man")
).expect("No files were found.");

// Return only Gzipped files using regular expression.
let files = Vec::<PathBuf>::try_from(
    Dowser::regex(r"(?i)[^/]+\.gz$").with_path("/usr/share/man")
).expect("No files were found.");

// Return only Gzipped files using callback filter.
let files = Vec::<PathBuf>::try_from(
    Dowser::filtered(|p: &Path| p.extension()
        .map_or(
            false,
            |e| e.as_bytes().eq_ignore_ascii_case(b"gz")
        )
    )
    .with_path("/usr/share/man")
).expect("No files were found.");
```
*/

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(macro_use_extern_crate)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]

#![allow(clippy::module_name_repetitions)] // This is fine.



mod ext;
mod dowse;
mod dowser;
mod hash;
pub mod utility;

pub use dowse::dowse;
pub use self::dowser::{
    Dowser,
    DowserError,
};
pub use ext::Extension;

#[doc(hidden)]
pub(crate) use hash::{
	NoHashU64,
	NoHashState,
};



#[doc(hidden)]
#[macro_export]
/// Helper: Mutex Unlock.
///
/// This just moves tedious code out of the way.
macro_rules! mutex_ptr {
	($mutex:expr) => (
		$mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
	);
}
