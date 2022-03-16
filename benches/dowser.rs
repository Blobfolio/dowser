/*!
# Benchmark: `dowser`
*/

use brunch::{
	Bench,
	benches,
};
use dowser::Dowser;
use std::{
	path::Path,
	time::Duration,
};

benches!(
	Bench::new("dowser::Dowser", "from(/usr/share)")
		.timed(Duration::from_secs(6))
		.with(|| Dowser::from(Path::new("/usr/share")).collect::<Vec<_>>()),
);
