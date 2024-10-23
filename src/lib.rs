/*!
# Dowser

[![docs.rs](https://img.shields.io/docsrs/dowser.svg?style=flat-square&label=docs.rs)](https://docs.rs/dowser/)
[![changelog](https://img.shields.io/crates/v/dowser.svg?style=flat-square&label=changelog&color=9b59b6)](https://github.com/Blobfolio/dowser/blob/master/CHANGELOG.md)<br>
[![crates.io](https://img.shields.io/crates/v/dowser.svg?style=flat-square&label=crates.io)](https://crates.io/crates/dowser)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/dowser/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/dowser/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/dowser/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/dowser)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/dowser/issues)

[`Dowser`] is a(nother) fast, recursive file-finding library for Unix/Rust. It differs from [`Walkdir`](https://crates.io/crates/walkdir) and kin in a number of ways:

* It is not limited to one root; any number of file and directory paths can be loaded and traversed en masse;
* Symlinks and hidden directories are followed like any other, including across devices;
* Matching file paths are canonicalized and deduped before yielding;

If those things sound nice, this library might be a good fit.

On the other hand, [`Dowser`] is optimized for _file_ searching; the iterator crawls but does not yield directory paths, which could be bad if you need those too. Haha.



## Example

All you need to do is chain [`Dowser::default`] with one or more of the following seed methods:

* [`Dowser::with_path`] / [`Dowser::with_paths`]
* [`Dowser::without_path`] / [`Dowser::without_paths`]

From there, you can apply any [`Iterator`](std::iter::Iterator) methods you want, or immediately collect the results using [`Dowser::into_vec`] or [`Dowser::into_vec_filtered`].

```
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
*/

#![forbid(unsafe_code)]

#![deny(
	clippy::allow_attributes_without_reason,
	clippy::correctness,
	unreachable_pub,
)]

#![warn(
	clippy::complexity,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::style,

	clippy::allow_attributes,
	clippy::clone_on_ref_ptr,
	clippy::create_dir,
	clippy::filetype_is_file,
	clippy::format_push_string,
	clippy::get_unwrap,
	clippy::impl_trait_in_params,
	clippy::lossy_float_literal,
	clippy::missing_assert_message,
	clippy::missing_docs_in_private_items,
	clippy::needless_raw_strings,
	clippy::panic_in_result_fn,
	clippy::pub_without_shorthand,
	clippy::rest_pat_in_fully_bound_structs,
	clippy::semicolon_inside_block,
	clippy::str_to_string,
	clippy::string_to_string,
	clippy::todo,
	clippy::undocumented_unsafe_blocks,
	clippy::unneeded_field_pattern,
	clippy::unseparated_literal_suffix,
	clippy::unwrap_in_result,

	macro_use_extern_crate,
	missing_copy_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]

#![expect(clippy::redundant_pub_crate, reason = "Unresolvable.")]

mod entry;
mod ext;
mod iter;

pub(crate) use entry::Entry;
pub use ext::Extension;
pub use iter::Dowser;
