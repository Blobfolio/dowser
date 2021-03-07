/*!
# Dowser: Filtered Find
*/

use dowser::Dowser;
use std::{
	convert::TryFrom,
	os::unix::ffi::OsStrExt,
	path::{
		Path,
		PathBuf,
	},
};

/// Do it.
fn main() {
	// Search for gzipped MAN pages.
	let files = Vec::<PathBuf>::try_from(
		Dowser::filtered(|p: &Path| p.extension()
			.map_or(
				false,
				|e| e.as_bytes().eq_ignore_ascii_case(b"gz")
			)
		)
		.with_path("/usr/share/man")
	).expect("No files were found.");

	println!("There are {} .gz files in /usr/share/man.", files.len());
}
