/*!
# Benchmark: `dowser::Extension`
*/

use brunch::{
	Bench,
	benches,
};
use dowser::Extension;
use std::{
	os::unix::ffi::OsStrExt,
	path::Path,
};

const JPG_ARR: &[u8] = b"jpg";
const JPG_EXT: Extension = Extension::new3(*b"jpg");

/// # Standard.
fn test_std<P>(path: P) -> bool
where P: AsRef<Path> {
	path.as_ref()
		.extension()
		.is_some_and(|p| p.as_bytes().eq_ignore_ascii_case(JPG_ARR))
}

/// # Dowser.
fn test_dowser<P>(path: P) -> bool
where P: AsRef<Path> {
	Some(JPG_EXT) == Extension::try_from3(path)
}


benches!(
	Bench::new("dowser::Extension::try_from3(/usr/share/image.jpg)::eq(JPG)")
		.run(|| test_dowser("/usr/share/image.jpg")),

	Bench::new("std::path::extension(/usr/share/image.jpg)::eq(JPG)")
		.run(|| test_std("/usr/share/image.jpg")),

	Bench::spacer(),

	Bench::new("dowser::Extension::slice_ext3(/usr/share/image.jpg)::eq(JPG)")
		.run(|| Extension::slice_ext3(b"/usr/share/image.jpg")),

	Bench::new("dowser::Extension::slice_ext(/usr/share/image.jpg)")
		.run(|| Extension::slice_ext(b"/usr/share/image.jpg")),

	Bench::new("std::path::extension(/usr/share/image.jpg)")
		.run(|| Path::new("/usr/share/image.jpg").extension()),
);
