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

 * Aside from [`Dowser::with_filter`] and [`Dowser::with_regex`], there is no way to alter its traversal behaviors;
 * There are no settings for things like min/max depth, directory filtering, etc.;
 * It only returns *file* paths. Directories are crawled, but not returned in the set;
 * File uniqueness hashing relies on Unix metadata; **this library is not compatible with Windows**;

Depending on your needs, those limitations could be bad, in which case something like [`Walkdir`](https://crates.io/crates/walkdir) might make more sense.



## Installation

Add `dowser` to your `dependencies` in `Cargo.toml`, like:

```
[dependencies]
dowser = "0.1.*"
```



## Features

| Feature | Description |
| ------- | ----------- |
| `regexp` | Enable the [`Dowser::with_regex`] method, which allows for matching file paths (as bytes) against a regular expression. |

To use this feature, alter the `Cargo.toml` bit to read:

```
[dependencies.dowser]
version = "0.1.*"
features = [ "regexp" ]
```



## Example

This crate comes with two ways to find files. If you already have the full list of starting path(s) and just want *all the files* that exist under them, use the [`dowse`](self::dowse()) method:

```rust
use std::path::PathBuf;

let paths = [ "/path/one", "/path/two", "/path/three" ];
let files: Vec<PathBuf> = dowser::dowse(&paths);
```

If you need to load starting paths multiple times, or want to filter the results, you can use the full [`Dowser`] struct instead. It follows a basic builder pattern, so you can just chain your way to an answer:

```rust
use dowser::Dowser;
use std::{
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

// Return all files under "/usr/share/man".
let res: Vec<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .build();

// Return only Gzipped files, using a regular expression.
// This requires the "regexp" feature.
let res: Vec<PathBuf> = Dowser::default()
    .with_regex(r"(?i)[^/]+\.gz$")
    .with_path("/usr/share/man")
    .build();

// The same thing, done manually.
let res: Vec<PathBuf> = Dowser::default()
    .with_filter(|p: &Path| p.extension()
        .map_or(
            false,
            |e| e.as_bytes().eq_ignore_ascii_case(b"gz")
        )
    )
    .with_path("/usr/share/man")
    .build();
```

If you want to easily bubble an error in cases where no files are found, you can use the [`std::convert::TryFrom`] trait (instead of calling [`Dowser::build`]), like:

```rust
use dowser::Dowser;
use std::convert::TryFrom;

let out = Vec::<PathBuf>::try_from(
    Dowser::default().with_path("/usr/share/man")
)
    .map_err(|_| YourErrorHere)?;
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

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



mod dowse;
mod dowser;
pub mod utility;

pub use dowse::dowse;
pub use dowser::Dowser;



#[doc(hidden)]
/// # (Not) Random State.
///
/// Using a fixed seed value for `AHashSet`/`AHashMap` drops a few dependencies
/// and prevents Valgrind complaining about 64 lingering bytes from the runtime
/// static that would be used otherwise.
///
/// For our purposes, the variability of truly random keys isn't really needed.
pub(crate) const AHASH_STATE: ahash::RandomState = ahash::RandomState::with_seeds(13, 19, 23, 71);



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
