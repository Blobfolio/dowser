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


const GZ: Extension = Extension::new2(*b"gz");


benches!(
	Bench::new("dowser::Dowser::from(/usr/share)")
		.run(|| Dowser::from("/usr/share").collect::<Vec<_>>()),

	Bench::spacer(),

	Bench::new("dowser::Dowser::from(/usr/share)::filter(gz)")
		.run_seeded(
			Dowser::from("/usr/share"),
			|d| d.filter(|p| Some(GZ) == Extension::try_from2(p)).collect::<Vec<_>>()
		),

	Bench::new("dowser::Dowser::from(/usr/share)::into_vec(gz)")
		.run_seeded(
			Dowser::from("/usr/share"),
			|d| d.into_vec(|p| Some(GZ) == Extension::try_from2(p))
		),
);
