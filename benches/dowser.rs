/*!
# Benchmark: `dowser`
*/

use brunch::{
	Bench,
	benches,
};
use dowser::Dowser;
use std::{
	path::PathBuf,
	time::Duration,
};

benches!(
	Bench::new("dowser::Dowser", "with_path(/usr/share)")
		.timed(Duration::from_secs(6))
		.with(|| Vec::<PathBuf>::try_from(Dowser::default().with_path("/usr/share")))
);
