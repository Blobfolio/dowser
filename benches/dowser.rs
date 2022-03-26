/*!
# Benchmark: `dowser`
*/

use brunch::{
	Bench,
	benches,
};
use dowser::{
	DirConcurrency,
	Dowser,
	Extension,
};
use std::{
	path::Path,
	time::Duration,
};


const GZ: Extension = Extension::new2(*b"gz");


benches!(
	Bench::new("dowser::Dowser", "from(/usr/share)")
		.timed(Duration::from_secs(6))
		.with(|| Dowser::from(Path::new("/usr/share")).collect::<Vec<_>>()),

	Bench::new("dowser::Dowser::from(/usr/share)", "with_dir_concurrency(Single)")
		.timed(Duration::from_secs(6))
		.with(|| Dowser::from(Path::new("/usr/share")).with_dir_concurrency(DirConcurrency::Single).collect::<Vec<_>>()),

	Bench::spacer(),

	Bench::new("dowser::Dowser::from(/usr/share)", "filter(gz)")
		.timed(Duration::from_secs(6))
		.with_setup(
			Dowser::from(Path::new("/usr/share")),
			|d| d.filter(|p| Some(GZ) == Extension::try_from2(p)).collect::<Vec<_>>()
		),

	Bench::new("dowser::Dowser::from(/usr/share)", "into_vec(gz)")
		.timed(Duration::from_secs(6))
		.with_setup(
			Dowser::from(Path::new("/usr/share")),
			|d| d.into_vec(|p| Some(GZ) == Extension::try_from2(p))
		),
);
