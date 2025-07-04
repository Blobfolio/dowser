/*!
# Dowser: Filtered Find
*/

/// # Do it.
fn main() {
	use dowser::{
		Dowser,
		Extension,
	};
	use std::{
		path::PathBuf,
		time::Instant,
	};

	const EXT: Extension = Extension::new("gz").unwrap();

	if std::fs::metadata("/usr/share").is_ok_and(|m| m.file_type().is_dir()) {
		// Search for gzipped MAN pages.
		let now = Instant::now();
		let files: Vec<PathBuf> = Dowser::default()
			.with_path("/usr/share")
			.filter(|p| EXT.matches_path(p))
			.collect();

		println!("Search took {} seconds.", now.elapsed().as_millis() as f64 / 1000.000);

		// Show what we found.
		if files.is_empty() {
			println!("No .gz files were found in /usr/share/.");
		}
		else {
			println!("There are {} .gz files in /usr/share/.", files.len());
		}
	}
	else {
		println!("Missing directory /usr/share.");
	}
}
