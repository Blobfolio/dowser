# Dowser

[![Documentation](https://docs.rs/dowser/badge.svg)](https://docs.rs/dowser/)
[![Changelog](https://img.shields.io/crates/v/dowser.svg?label=Changelog&color=9cf)](https://github.com/Blobfolio/dowser/blob/master/CHANGELOG.md)
[![crates.io](https://img.shields.io/crates/v/dowser.svg)](https://crates.io/crates/dowser)
[![Build Status](https://github.com/Blobfolio/dowser/workflows/Build/badge.svg)](https://github.com/Blobfolio/dowser/actions)
[![Dependency Status](https://deps.rs/repo/github/blobfolio/dowser/status.svg)](https://deps.rs/repo/github/blobfolio/dowser)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](https://github.com/Blobfolio/dowser)

`Dowser` is a(nother) fast, recursive file-finding library for Unix/Rust. It differs from [`Walkdir`](https://crates.io/crates/walkdir) and kin in a number of ways:

* It is not limited to one root; any number of file and directory paths can be loaded and traversed en masse;
* Symlinks and hidden directories are followed like any other, including across devices;
* Matching file paths are canonicalized and deduped before yielding;

If those things sound nice, this library might be a good fit.

On the other hand, `Dowser` is optimized for _file_ searching; the iterator crawls but does not yield directory paths, which could be bad if you need those too. Haha.



## Installation

Add `dowser` to your `dependencies` in `Cargo.toml`, like:

```
[dependencies]
dowser = "0.6.*"
```



## Example

All you need to do is chain `Dowser::default` with one or more of the following seed methods:

* `Dowser::with_path` / `Dowser::with_paths`
* `Dowser::without_path` / `Dowser::without_paths`

From there, you can apply any `Iterator` methods you want, or immediately collect the results using `Dowser::into_vec` or `Dowser::into_vec_filtered`.

```rust
use dowser::Dowser;
use std::path::PathBuf;

// Return all files under "/usr/share/man".
let files1: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .collect();

// Same as above, but slightly faster.
let files2: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .into_vec();

assert_eq!(files1.len(), files2.len());

// Return only Gzipped files using callback filter.
let files1: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .filter(|p|
        p.extension().map_or(
            false,
            |e| e.eq_ignore_ascii_case("gz")
        )
    )
    .collect();

// Same as above, but slightly faster.
let files2: Vec::<PathBuf> = Dowser::default()
    .with_path("/usr/share/man")
    .into_vec_filtered(|p|
        p.extension().map_or(
            false,
            |e| e.eq_ignore_ascii_case("gz")
        )
    );

assert_eq!(files1.len(), files2.len());
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
