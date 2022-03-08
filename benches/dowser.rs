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
	Bench::new("dowser::Dowser", "from(/usr/share)")
		.timed(Duration::from_secs(6))
		.with(|| Dowser::from(PathBuf::from("/usr/share")).collect::<Vec<_>>()),
);
