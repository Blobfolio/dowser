/*!
# Benchmark: `dowser`
*/

use brunch::{
	Bench,
	benches,
};
use dowser::{
	Dowser,
	Extension,
};


const GZ: Extension = Extension::new("gz").unwrap();


benches!(
	Bench::new("dowser::Dowser::from(/usr/share).collect()")
		.run(|| Dowser::from("/usr/share").collect::<Vec<_>>()),

	Bench::spacer(),

	Bench::new("dowser::Dowser::from(/usr/share).filter(gz).collect()")
		.run_seeded(
			Dowser::from("/usr/share"),
			|d| d.filter(|p| GZ.matches_path(p)),
		),
);
