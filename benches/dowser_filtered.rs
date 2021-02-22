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
	Bench::new("dowser::Dowser", "with_filter(.gz)")
		.with(|| Dowser::default().with_filter(cb).with_path("/usr/share/man").build()),

	Bench::new("dowser::Dowser", "with_regex(.gz)")
		.with(|| Dowser::default().with_regex(r"(?i).+\.gz$").with_path("/usr/share/man").build())
);

#[cfg(not(feature = "regexp"))]
benches!(
	Bench::new("dowser::Dowser", "with_filter(.gz)")
		.with(|| Dowser::default().with_filter(cb).with_path("/usr/share/man").build())
);
