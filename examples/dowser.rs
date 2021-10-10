/*!
# Dowser: Filtered Find
*/

use dowser::{
	Dowser,
	Extension,
};
use std::{
	path::{
		Path,
		PathBuf,
	},
};

/// Do it.
fn main() {
	const EXT: Extension = Extension::new2(*b"gz");

	// Search for gzipped MAN pages.
	let files = Vec::<PathBuf>::try_from(
		Dowser::filtered(|p: &Path| Extension::try_from2(p).map_or(false, |p| p == EXT))
		.with_path("/usr/share/man")
	).expect("No files were found.");

	println!("There are {} .gz files in /usr/share/man.", files.len());
}
