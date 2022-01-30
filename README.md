# Dowser

[![Documentation](https://docs.rs/dowser/badge.svg)](https://docs.rs/dowser/)
[![crates.io](https://img.shields.io/crates/v/dowser.svg)](https://crates.io/crates/dowser)
[![Build Status](https://github.com/Blobfolio/dowser/workflows/Build/badge.svg)](https://github.com/Blobfolio/dowser/actions)
[![Dependency Status](https://deps.rs/repo/github/blobfolio/dowser/status.svg)](https://deps.rs/repo/github/blobfolio/dowser)

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

```
[dependencies]
dowser = "0.3.*"
```



## Features

| Feature | Description | Default |
| ------- | ----------- | ------- |
| `parking_lot_mutex` | Use [`parking_lot::Mutex`] instead of [`std::sync::Mutex`]. | Y |
| `regexp` | Enable the [`Dowser::regex`] method, which allows for matching file paths (as bytes) against a regular expression. | N |

To use this feature, alter the `Cargo.toml` bit to read:

```
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



## License

See also: [CREDITS.md](CREDITS.md)

Copyright © 2022 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

This work is free. You can redistribute it and/or modify it under the terms of the Do What The Fuck You Want To Public License, Version 2.

    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    Version 2, December 2004
    
    Copyright (C) 2004 Sam Hocevar <sam@hocevar.net>
    
    Everyone is permitted to copy and distribute verbatim or modified
    copies of this license document, and changing it is allowed as long
    as the name is changed.
    
    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
    
    0. You just DO WHAT THE FUCK YOU WANT TO.
