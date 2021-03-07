/*!
# Benchmark: `dowser::Dowser` (Filtered)
*/

use brunch::{
	Bench,
	benches,
};
use dowser::Dowser;
use std::{
	convert::TryFrom,
	os::unix::ffi::OsStrExt,
	path::{
		Path,
		PathBuf,
	}
};

/// # Filter Callback.
fn cb(path: &Path) -> bool {
	path.extension()
		.map_or(
			false,
			|e| e.as_bytes().eq_ignore_ascii_case(b"gz")
		)
}

#[cfg(feature = "regexp")]
benches!(
	Bench::new("dowser::Dowser", "filtered(.gz)")
		.with(|| Vec::<PathBuf>::try_from(
			Dowser::filtered(cb).with_path("/usr/share/man")
		)),

	Bench::new("dowser::Dowser", "regex(.gz)")
		.with(|| Vec::<PathBuf>::try_from(
			Dowser::regex(r"(?i).+\.gz$").with_path("/usr/share/man")
		))
);

#[cfg(not(feature = "regexp"))]
benches!(
	Bench::new("dowser::Dowser", "filtered(.gz)")
		.with(|| Vec::<PathBuf>::try_from(
			Dowser::filtered(cb).with_path("/usr/share/man")
		))
);
