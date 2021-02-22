/*!
# Dowser: Filtered Find
*/

use std::{
	os::unix::ffi::OsStrExt,
	path::Path,
};

/// Do it.
fn main() {
	// Search for gzipped MAN pages.
	let len: usize = dowser::Dowser::default()
		.with_filter(|p: &Path| p.extension()
			.map_or(
				false,
				|e| e.as_bytes().eq_ignore_ascii_case(b"gz")
			)
		)
		.with_path("/usr/share/man")
		.build()
		.len();

	println!("There are {} .gz files in /usr/share/man.", len);
}
