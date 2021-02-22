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

This crate comes with two ways to find files. If you already have the full list of starting path(s) and just want *all the files* that exist under them, use the [`dowse`] method:

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
    .with_regex(r"(?i)[^/]+\.gz$") // Filter before adding paths!
    .with_path("/usr/share/man")
    .build();

// The same thing, done manually.
let res: Vec<PathBuf> = Dowser::default()
    .with_filter(|p: &Path| p.extension()
        .map_or(
            false,
            |e| e.as_bytes().eq_ignore_ascii_case(b"gz")
        )
    ) // Again, filter before adding paths!
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



## License

See also: [CREDITS.md](CREDITS.md)

Copyright © 2021 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

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
