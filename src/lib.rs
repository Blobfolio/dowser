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
dowser = "0.3.*"
```



## Features

| Feature | Description | Default |
| ------- | ----------- | ------- |
| `parking_lot_mutex` | Use [`parking_lot::Mutex`] instead of [`std::sync::Mutex`]. | Y |
| `regexp` | Enable the [`Dowser::regex`] method, which allows for matching file paths (as bytes) against a regular expression. | N |

To use this feature, alter the `Cargo.toml` bit to read:

```ignore
[dependencies.dowser]
version = "0.3.*"
features = [ "regexp" ]
```



## Example

If you want to filter files or need to add path(s) to the crawl list multiple times, initialize a [`Dowser`] object with one of the following three methods:

 * [`Dowser::default`] Return all files without prejudice.
 * [`Dowser::filtered`] Filter file paths via the provided callback.
 * [`Dowser::regex`] Filter file paths via regular express. (This requires enabling the `regexp` crate feature.)

From there, add one or more file or directory paths using the [`Dowser::with_path`] and [`Dowser::with_paths`] methods.

Finally, collect the results with `Vec::<PathBuf>::try_from()`. If no files are found, an error is returned, otherwise the matching file paths are collected into a vector.

```rust
use dowser::Dowser;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

// Return all files under "/usr/share/man".
let files = Vec::<PathBuf>::try_from(
   Dowser::default().with_path("/usr/share/man")
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

If compiled with the `regexp` flag, you can additionally filter by regular
expression:

```no_run,ignore
use dowser::Dowser;
use std::path::PathBuf;

// Return only Gzipped files using regular expression.
let files = Vec::<PathBuf>::try_from(
    Dowser::regex(r"(?i)[^/]+\.gz$").with_path("/usr/share/man")
).expect("No files were found.");
```

Note: If you want a vector back no matter what — even if empty — you can
use [`Dowser::into_vec`] instead of `TryFrom::<PathBuf>`.
*/

#![forbid(unsafe_code)]

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
mod hash;
mod iter;
pub mod utility;



pub use iter::Dowser;
pub use ext::Extension;

#[doc(hidden)]
pub(crate) use hash::{
	NoHashU64,
	NoHashState,
};
