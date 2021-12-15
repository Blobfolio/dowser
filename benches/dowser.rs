/*!
# Benchmark: `dowser`
*/

use brunch::{
	Bench,
	benches,
};
use dowser::Dowser;
use std::time::Duration;

benches!(
	Bench::new("dowser::Dowser", "with_path(/usr/share)")
		.timed(Duration::from_secs(6))
		.with(|| Dowser::default().with_path("/usr/share").into_vec())
);
