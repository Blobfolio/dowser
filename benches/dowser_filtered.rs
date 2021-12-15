/*!
# Benchmark: `dowser::Dowser` (Filtered)
*/

use brunch::{
	Bench,
	benches,
};
use dowser::Dowser;
use std::{
	os::unix::ffi::OsStrExt,
	path::Path,
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
		.with(|| Dowser::filtered(cb).with_path("/usr/share/man").into_vec()),

	Bench::new("dowser::Dowser", "regex(.gz)")
		.with(|| Dowser::regex(r"(?i).+\.gz$").with_path("/usr/share/man").into_vec())
);

#[cfg(not(feature = "regexp"))]
benches!(
	Bench::new("dowser::Dowser", "filtered(.gz)")
		.with(|| Dowser::filtered(cb).with_path("/usr/share/man").into_vec())
);
