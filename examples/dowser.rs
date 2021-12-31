/*!
# Dowser: Filtered Find
*/

use dowser::{
	Dowser,
	Extension,
};
use std::{
	path::Path,
	time::Instant,
};

/// Do it.
fn main() {
	const EXT: Extension = Extension::new2(*b"gz");

	// Search for gzipped MAN pages.
	let now = Instant::now();
	let files = Dowser::filtered(|p: &Path| Extension::try_from2(p).map_or(false, |p| p == EXT))
		.with_path("/usr/share")
		.into_vec();

	println!("Search took {} seconds.", now.elapsed().as_millis() as f64 / 1000.0);

	// Show what we found.
	if files.is_empty() {
		println!("No .gz files were found in /usr/share/.");
	}
	else {
		println!("There are {} .gz files in /usr/share/.", files.len());
	}
}
