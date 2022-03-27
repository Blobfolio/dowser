/*!
# Dowser

[`Dowser`] is a(nother) fast, multi-threaded, recursive file-finding library for Unix/Rust. It differs from [`Walkdir`](https://crates.io/crates/walkdir) and kin in a number of ways:

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
dowser = "0.4.*"
```



## Features

| Feature | Description | Default |
| ------- | ----------- | ------- |
| `parking_lot_mutex` | Use [`parking_lot::Mutex`] instead of [`std::sync::Mutex`]. | Y |

To use this feature, alter the `Cargo.toml` bit to read:

```text,ignore
[dependencies.dowser]
version = "0.4.*"
features = [ "parking_lot_mutex" ]
```



## Example

All you need to do is chain [`Dowser::default`] with one or more of the
following seed methods:

* [`Dowser::with_path`] / [`Dowser::with_paths`]
* [`Dowser::without_path`] / [`Dowser::without_paths`]

From there, you can use whatever `Iterator` methods you want.

```
use dowser::{
    Dowser,
    Extension,
};
use std::os::unix::ffi::OsStrExt;
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
            |e| e.as_bytes().eq_ignore_ascii_case(b"gz")
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



pub use iter::{
	DirConcurrency,
	Dowser,
};
pub use ext::Extension;

#[doc(hidden)]
pub(crate) use hash::NoHashState;
