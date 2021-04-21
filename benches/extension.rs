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
	time::Duration,
};

const JPG_ARR: &[u8] = b"jpg";
const JPG_EXT: Extension = Extension::new3(*b"jpg");

/// # Standard.
fn test_std<P>(path: P) -> bool
where P: AsRef<Path> {
	path.as_ref()
		.extension()
		.map_or(
			false,
			|p| p.as_bytes().to_ascii_lowercase() == JPG_ARR
		)
}

/// # Dowser.
fn test_dowser<P>(path: P) -> bool
where P: AsRef<Path> {
	Extension::try_from3(path)
		.map_or(false, |p| p == JPG_EXT)
}


benches!(
	Bench::new("dowser::Extension", "try_from3(/usr/share/image.jpg)")
		.timed(Duration::from_secs(2))
		.with(|| test_dowser("/usr/share/image.jpg")),

	Bench::new("std::path", "extension(/usr/share/image.jpg)")
		.timed(Duration::from_secs(2))
		.with(|| test_std("/usr/share/image.jpg"))
);
