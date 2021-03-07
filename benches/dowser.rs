/*!
# Benchmark: `dowser`
*/

use brunch::{
	Bench,
	benches,
};
use dowser::{
	dowse,
	Dowser,
};
use std::{
	convert::TryFrom,
	path::PathBuf,
	time::Duration,
};

benches!(
	Bench::new("dowser", "dowse(/usr/share)")
		.timed(Duration::from_secs(5))
		.with(|| dowse(&["/usr/share"])),

	Bench::new("dowser::Dowser", "with_path(/usr/share)")
		.timed(Duration::from_secs(6))
		.with(|| Vec::<PathBuf>::try_from(Dowser::default().with_path("/usr/share")))
);
