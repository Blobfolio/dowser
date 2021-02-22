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
use std::time::Duration;

benches!(
	Bench::new("dowser", "dowse(/usr/share)")
		.timed(Duration::from_secs(5))
		.with(|| dowse(&["/usr/share"])),

	Bench::new("dowser::Dowser", "with_path(/usr/share)")
		.timed(Duration::from_secs(5))
		.with(|| Dowser::default().with_path("/usr/share").build())
);
