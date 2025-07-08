/*!
# Benchmark: `dowser::Extension`
*/

use brunch::{
	Bench,
	benches,
};

use dowser::Extension;
use std::path::Path;


const EXT_JPEG: Extension = Extension::new("jpg").unwrap();


benches!(
	Bench::new("dowser::Extension::from_path()")
		.run(|| Extension::from_path("/usr/share/image.jpg")),

	Bench::new("dowser::Extension::from_path_slice()")
		.run(|| Extension::from_path_slice(b"/usr/share/image.jpg")),

	Bench::new("std::path::Path::new()::extension()")
		.run(|| Path::new("/usr/share/image.jpg").extension()),

	Bench::spacer(),

	Bench::new("<EXT_JPEG>::matches_path()")
		.run(|| EXT_JPEG.matches_path("/usr/share/image.jpg")),

	Bench::new("<EXT_JPEG>::matches_path_slice()")
		.run(|| EXT_JPEG.matches_path_slice(b"/usr/share/image.jpg")),

	Bench::new("dowser::Extension::from_path_slice() == Some(EXT_JPEG)")
		.run(|| matches!(Extension::from_path_slice(b"/usr/share/image.jpg"), Some(EXT_JPEG))),

	Bench::new("std::path::Path::new()::extension()::eq_ignore_ascii_case()")
		.run(||
			Path::new("/usr/share/image.jpg").extension()
				.is_some_and(|o| o.eq_ignore_ascii_case("jpg"))
		),
);
